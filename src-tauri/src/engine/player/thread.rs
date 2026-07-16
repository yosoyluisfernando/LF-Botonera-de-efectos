//! Modulo: engine/player/thread.rs
//! Proposito: hilo dedicado del motor del reproductor. Dueno de los dos decks;
//! traduce comandos y el fin natural de pista en acciones de la cola
//! (`QueueState`) y las ejecuta sobre ellos. Sondea el fin del deck activo para
//! el avance ping-pong.
//!
//! Ya NO posee tarjeta: entrega sus fuentes al bus `Reproductor` de la consola,
//! como el motor de efectos entrega las suyas a los suyos. Sigue siendo un motor
//! independiente —su cola, su avance, su transporte— pero comparte la salida.
use super::deck::{Deck, DeckStatus};
use super::exec::exec_all;
use super::queue::{DeckAction, QueueState};
use super::resolve::QueueResolver;
use super::{PlayerCommand, PlayerSnapshot};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::{BusId, ConsoleEngine, Routing};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

struct Motor {
    decks: Vec<Deck>,
    queue: QueueState,
}

pub fn run(
    rx: Receiver<PlayerCommand>,
    cache: Arc<Mutex<PreloadCache>>,
    volume: Arc<AtomicU32>,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
    resolver: QueueResolver,
    console: Arc<ConsoleEngine>,
) {
    // Los dos decks del ping-pong. Se crean una vez: ya no dependen de que haya
    // tarjeta, porque no la abren — entregan al bus cuando cargan.
    let mut motor = Motor {
        decks: vec![Deck::new(), Deck::new()],
        queue: QueueState::new(),
    };
    loop {
        let sync = match rx.recv_timeout(TICK) {
            Ok(PlayerCommand::Sync(done)) => Some(done),
            Ok(cmd) => {
                handle(cmd, &mut motor, &cache, &resolver, &console);
                None
            }
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let active = motor.queue.active_deck();
        if motor.decks.get_mut(active).is_some_and(|d| d.poll_finished()) {
            let actions = motor.queue.advance(false);
            let bus = console.bus(BusId::Reproductor);
            exec_all(actions, &mut motor.decks, bus.as_ref(), &cache, &resolver);
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
    resolver: &QueueResolver,
    console: &Arc<ConsoleEngine>,
) {
    let actions = match cmd {
        PlayerCommand::SetRouting(routing) => {
            set_routing(routing, motor, console);
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
        PlayerCommand::Sync(_) => return,
    };
    let bus = console.bus(BusId::Reproductor);
    exec_all(actions, &mut motor.decks, bus.as_ref(), cache, resolver);
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

/// Cambia por donde sale el reproductor. `Routing::Program` = suma en el
/// programa, asi que obedece al master; `Device(x)` = sale por su tarjeta, ajeno
/// a el.
///
/// Reconstruir el grafo mata las fuentes que estuvieran sonando en ese bus, asi
/// que el transporte se detiene: no deben sobrevivir indices "sonando" apuntando
/// a fuentes que ya no existen. La Fase 4.5 lo hara en caliente reconstruyendolas
/// en su posicion.
fn set_routing(routing: Routing, motor: &mut Motor, console: &Arc<ConsoleEngine>) {
    let _ = motor.queue.stop();
    for deck in motor.decks.iter_mut() {
        deck.stop();
    }
    console.set_bus_routing(BusId::Reproductor, routing);
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
