//! Modulo: engine/player/thread.rs
//! Proposito: hilo dedicado del motor del reproductor. Unico dueno del
//! OutputStream propio y los dos decks (rodio no es Send). Traduce comandos y el
//! fin natural de pista en acciones de la cola (`QueueState`) y las ejecuta sobre
//! los decks. Sondea el fin del deck activo para el avance ping-pong.
use super::deck::{Deck, DeckStatus};
use super::exec::exec_all;
use super::queue::{DeckAction, QueueState};
use super::resolve::QueueResolver;
use super::{PlayerCommand, PlayerSnapshot};
use crate::engine::console::device::find_device;
use crate::engine::cache::preload::PreloadCache;
use rodio::{OutputStream, OutputStreamHandle};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

struct Motor {
    stream: Option<(OutputStream, OutputStreamHandle)>,
    decks: Vec<Deck>,
    queue: QueueState,
}

pub fn run(
    rx: Receiver<PlayerCommand>,
    cache: Arc<Mutex<PreloadCache>>,
    volume: Arc<AtomicU32>,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
    resolver: QueueResolver,
) {
    let mut motor = Motor { stream: None, decks: Vec::new(), queue: QueueState::new() };
    loop {
        let sync = match rx.recv_timeout(TICK) {
            Ok(PlayerCommand::Sync(done)) => Some(done),
            Ok(cmd) => {
                handle(cmd, &mut motor, &cache, &volume, &resolver);
                None
            }
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let active = motor.queue.active_deck();
        if motor.decks.get_mut(active).is_some_and(|d| d.poll_finished()) {
            let actions = motor.queue.advance(false);
            exec_all(actions, &mut motor.decks, &cache, &volume, &resolver);
        }
        refresh(&motor, &snapshot, &volume);
        if let Some(done) = sync {
            let _ = done.send(());
        }
    }
}

fn handle(
    cmd: PlayerCommand,
    motor: &mut Motor,
    cache: &Arc<Mutex<PreloadCache>>,
    volume: &Arc<AtomicU32>,
    resolver: &QueueResolver,
) {
    let actions = match cmd {
        PlayerCommand::SetDevice(name) => {
            open_device(&name, motor);
            return;
        }
        PlayerCommand::SetQueue(entries) => motor.queue.set_entries(entries),
        PlayerCommand::SetMode(mode) => motor.queue.set_mode(mode),
        PlayerCommand::SetStopAfter(v) => {
            motor.queue.set_stop_after(v);
            return;
        }
        PlayerCommand::SetLoopCurrent(v) => {
            motor.queue.set_loop_current(v);
            return;
        }
        PlayerCommand::Seek(position_s) => motor.queue.seek(position_s),
        PlayerCommand::MarkNext(i) => motor.queue.mark_next(i),
        PlayerCommand::PlayIndex(i) => motor.queue.start_at(i),
        PlayerCommand::ActivateIndex(i) => {
            let playing = is_playing(motor);
            motor.queue.activate(i, playing)
        }
        PlayerCommand::Next => motor.queue.advance(true),
        PlayerCommand::Prev => motor.queue.prev(),
        PlayerCommand::Stop => motor.queue.stop(),
        PlayerCommand::Pause => {
            if let Some(d) = motor.decks.get_mut(motor.queue.active_deck()) {
                d.pause();
            }
            return;
        }
        PlayerCommand::Resume => resume(motor),
        PlayerCommand::SetVolume(v) => {
            for deck in motor.decks.iter() {
                deck.apply_volume(v);
            }
            return;
        }
        PlayerCommand::Sync(_) => return,
    };
    exec_all(actions, &mut motor.decks, cache, volume, resolver);
}

/// Reanudar: si el deck activo esta en pausa, sigue; si estaba detenido, arranca
/// la siguiente (respetando lo marcado, como pide "detener al finalizar").
/// ¿Suena algo ahora mismo? Lo dice el deck, no la cola: una pista huerfana
/// (borrada de la lista mientras sonaba) sigue sonando sin estar en la cola.
fn is_playing(motor: &Motor) -> bool {
    motor
        .decks
        .get(motor.queue.active_deck())
        .is_some_and(|d| d.status() == DeckStatus::Playing)
}

fn resume(motor: &mut Motor) -> Vec<DeckAction> {
    let active = motor.queue.active_deck();
    if motor.decks.get(active).map(|d| d.status()) == Some(DeckStatus::Paused) {
        if let Some(d) = motor.decks.get_mut(active) {
            d.play();
        }
        return Vec::new();
    }
    motor.queue.resume_next()
}

/// Abre (o reabre) el OutputStream propio en el dispositivo dado y recrea los dos
/// decks. Si el dispositivo no esta disponible, suelta el stream: la recuperacion
/// la maneja el flujo de dispositivos que ya existe en la botonera.
fn open_device(name: &str, motor: &mut Motor) {
    // Cambiar o perder la salida detiene el transporte: no deben sobrevivir
    // indices "sonando" asociados a decks que acaban de ser destruidos.
    let _ = motor.queue.stop();
    let Some(device) = find_device(name) else {
        motor.stream = None;
        motor.decks.clear();
        return;
    };
    match OutputStream::try_from_device(&device) {
        Ok((s, handle)) => {
            let mut fresh = Vec::new();
            for _ in 0..2 {
                if let Some(deck) = Deck::new(&handle) {
                    fresh.push(deck);
                }
            }
            if fresh.len() != 2 {
                motor.stream = None;
                motor.decks.clear();
                return;
            }
            motor.decks = fresh;
            motor.stream = Some((s, handle));
        }
        Err(_) => {
            motor.stream = None;
            motor.decks.clear();
        }
    }
}

fn refresh(motor: &Motor, snapshot: &Arc<Mutex<PlayerSnapshot>>, volume: &Arc<AtomicU32>) {
    let mut snap = PlayerSnapshot {
        volume: f32::from_bits(volume.load(Ordering::Relaxed)),
        current_index: motor.queue.current().map(|i| i as u32),
        next_index: motor.queue.next().map(|i| i as u32),
        mode: motor.queue.mode().as_str().to_string(),
        stop_after: motor.queue.stop_after(),
        loop_current: motor.queue.loop_current(),
        queue_len: motor.queue.len() as u32,
        ..Default::default()
    };
    if let Some(deck) = motor.decks.get(motor.queue.active_deck()) {
        snap.playing = deck.status() == DeckStatus::Playing;
        snap.path = deck.path().map(str::to_string);
        snap.position_s = deck.position_s();
        snap.duration_s = deck.duration_s();
        snap.can_seek = deck.can_seek();
    }
    *snapshot.lock().unwrap() = snap;
}
