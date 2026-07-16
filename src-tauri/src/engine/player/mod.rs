//! Modulo: engine/player/mod.rs
//! Proposito: motor propio del reproductor auxiliar (modo reproductor). Es
//! INDEPENDIENTE del motor de efectos: su propio hilo, su propio OutputStream y
//! dispositivo, y su propio volumen. Dos decks con pre-carga ping-pong. Reutiliza
//! `build_play_source` (cue/cache) y `find_device`; la decision de avance vive en
//! `domain::player`. Guia: Documentacion/PLAN_MODO_REPRODUCTOR.md.
mod deck;
mod exec;
pub mod monitor;
mod queue;
mod queue_edit;
mod queue_ops;
mod queue_select;
pub mod resolve;
mod thread;

pub use queue::QueueEntry;
pub use resolve::QueueResolver;

use crate::domain::player::PlayerMode;
use crate::engine::cache::preload::PreloadCache;
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, sync_channel, Sender};
use std::sync::{Arc, Mutex};

/// Comandos que el handle envia al hilo del motor.
pub enum PlayerCommand {
    SetDevice(String),
    SetQueue(Vec<QueueEntry>),
    SetMode(PlayerMode),
    SetStopAfter(bool),
    SetLoopCurrent(bool),
    Seek(f64),
    MarkNext(Option<usize>),
    PlayIndex(usize),
    ActivateIndex(usize),
    Next,
    Prev,
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
    Sync(std::sync::mpsc::SyncSender<()>),
}

/// Estado en vivo que lee la UI (solo pinta). Lo refresca el hilo cada tick.
/// `current_index` (verde) y `next_index` (naranja) apuntan a la cola.
/// `PartialEq` permite al monitor emitir solo cuando algo cambia.
#[derive(Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerSnapshot {
    pub playing: bool,
    pub path: Option<String>,
    pub position_s: f64,
    pub duration_s: f64,
    pub current_index: Option<u32>,
    pub next_index: Option<u32>,
    pub mode: String,
    pub stop_after: bool,
    /// Boton Loop activo: la cancion actual se repite hasta desactivarlo.
    pub loop_current: bool,
    /// La barra de progreso solo deja arrastrar si esto es cierto: una locucion
    /// son varios archivos encadenados y no se puede reposicionar.
    pub can_seek: bool,
    /// Volumen propio del reproductor (0.0..=1.5). Va en el tick para que el
    /// boton de volumen se pinte solo, sin tener que preguntar por su cuenta.
    pub volume: f32,
    pub queue_len: u32,
}

/// Handle del motor (Send + Sync). No toca audio; todo pasa por el canal al hilo
/// dedicado, unico dueno del OutputStream y los decks.
pub struct PlayerEngine {
    tx: Sender<PlayerCommand>,
    volume: Arc<AtomicU32>,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
}

impl PlayerEngine {
    /// Arranca el hilo del motor. `cache` se comparte con la precarga de efectos.
    /// `resolver` traduce los tipos especiales en el momento de sonar.
    pub fn new(cache: Arc<Mutex<PreloadCache>>, resolver: QueueResolver) -> Self {
        let (tx, rx) = channel::<PlayerCommand>();
        let volume = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        let snapshot = Arc::new(Mutex::new(PlayerSnapshot::default()));
        let volume_thread = Arc::clone(&volume);
        let snapshot_thread = Arc::clone(&snapshot);
        std::thread::spawn(move || {
            thread::run(rx, cache, volume_thread, snapshot_thread, resolver)
        });
        Self { tx, volume, snapshot }
    }

    fn send(&self, cmd: PlayerCommand) {
        let _ = self.tx.send(cmd);
    }

    pub fn set_device(&self, name: &str) {
        self.send(PlayerCommand::SetDevice(name.to_string()));
    }
    pub fn set_queue(&self, entries: Vec<QueueEntry>) {
        self.send(PlayerCommand::SetQueue(entries));
    }
    pub fn set_mode(&self, mode: PlayerMode) {
        self.send(PlayerCommand::SetMode(mode));
    }
    pub fn set_stop_after(&self, value: bool) {
        self.send(PlayerCommand::SetStopAfter(value));
    }
    /// Boton Loop: repetir la cancion actual hasta desactivarlo.
    pub fn set_loop_current(&self, value: bool) {
        self.send(PlayerCommand::SetLoopCurrent(value));
    }
    /// Salta a `position_s` de la pista que suena (relativo a su cue de inicio).
    pub fn seek(&self, position_s: f64) {
        self.send(PlayerCommand::Seek(position_s));
    }
    pub fn mark_next(&self, index: Option<usize>) {
        self.send(PlayerCommand::MarkNext(index));
    }
    pub fn play_index(&self, index: usize) {
        self.send(PlayerCommand::PlayIndex(index));
    }
    /// Doble clic en una fila: el motor decide segun suene o no (ver `activate`).
    pub fn activate_index(&self, index: usize) {
        self.send(PlayerCommand::ActivateIndex(index));
    }
    pub fn next(&self) {
        self.send(PlayerCommand::Next);
    }
    pub fn prev(&self) {
        self.send(PlayerCommand::Prev);
    }
    pub fn pause(&self) {
        self.send(PlayerCommand::Pause);
    }
    pub fn resume(&self) {
        self.send(PlayerCommand::Resume);
    }
    pub fn stop(&self) {
        self.send(PlayerCommand::Stop);
    }

    /// Fija el volumen propio (0.0..=1.5) y lo comunica al hilo.
    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.5);
        self.volume.store(clamped.to_bits(), Ordering::Relaxed);
        self.send(PlayerCommand::SetVolume(clamped));
    }
    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    /// Handle crudo del snapshot para el monitor. A diferencia de `snapshot()`,
    /// no impone la barrera FIFO: el monitor solo observa, no espera comandos.
    pub fn snapshot_handle(&self) -> Arc<Mutex<PlayerSnapshot>> {
        Arc::clone(&self.snapshot)
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        // Barrera FIFO: garantiza que el estado devuelto ya incluye todos los
        // comandos enviados antes de esta lectura (p. ej. una edicion de cola).
        let (done_tx, done_rx) = sync_channel(0);
        if self.tx.send(PlayerCommand::Sync(done_tx)).is_ok() {
            let _ = done_rx.recv_timeout(std::time::Duration::from_secs(1));
        }
        self.snapshot.lock().unwrap().clone()
    }
}
