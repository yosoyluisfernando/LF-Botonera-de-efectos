//! Modulo: engine/player/mod.rs
//! Proposito: motor propio del reproductor auxiliar (modo reproductor).
//!
//! Sigue siendo INDEPENDIENTE del motor de efectos en lo que importa: su hilo, su
//! cola, su avance y su transporte. El Stop general y el Solo de los efectos no
//! lo tocan. Lo que ya NO tiene es tarjeta propia: entrega sus fuentes al bus
//! `Reproductor` de la consola, que suma en el programa. Por eso obedece al
//! master (decision del autor, 2026-07-16) — en volumen, no en transporte.
//!
//! Su volumen es el fader de ese bus: bajar la musica para hablar encima es mover
//! ese fader, y no toca a los efectos. Dos decks con pre-carga ping-pong.
//! Guia: Documentacion/PLAN_MODO_REPRODUCTOR.md y PLAN_CONSOLA_VIRTUAL.md.
mod deck;
mod deck_track;
mod exec;
pub mod monitor;
mod prefetch;
mod queue;
mod queue_edit;
mod queue_ops;
mod queue_select;
pub mod resolve;
mod source;
mod thread;

pub use queue::QueueEntry;
pub use resolve::QueueResolver;

use crate::domain::player::PlayerMode;
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::{BusId, ConsoleEngine, Routing};
use serde::Serialize;
use std::sync::mpsc::{channel, sync_channel, Sender};
use std::sync::{Arc, Mutex};

/// Comandos que el handle envia al hilo del motor.
pub enum PlayerCommand {
    SetRouting(Routing),
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
/// dedicado, dueno de los decks.
pub struct PlayerEngine {
    tx: Sender<PlayerCommand>,
    console: Arc<ConsoleEngine>,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
}

impl PlayerEngine {
    /// Arranca el hilo del motor. `cache` se comparte con la precarga de efectos.
    /// `resolver` traduce los tipos especiales en el momento de sonar.
    pub fn new(
        cache: Arc<Mutex<PreloadCache>>,
        resolver: QueueResolver,
        console: Arc<ConsoleEngine>,
    ) -> Self {
        let (tx, rx) = channel::<PlayerCommand>();
        let snapshot = Arc::new(Mutex::new(PlayerSnapshot::default()));
        let snapshot_thread = Arc::clone(&snapshot);
        // El volumen del reproductor ES el fader de su bus: el hilo lo lee de la
        // consola para el snapshot, no lo guarda por su cuenta.
        let volume_thread = console.volume_handle(BusId::Reproductor);
        let console_thread = Arc::clone(&console);
        std::thread::spawn(move || {
            thread::run(
                rx,
                cache,
                volume_thread,
                snapshot_thread,
                resolver,
                console_thread,
            )
        });
        Self { tx, console, snapshot }
    }

    fn send(&self, cmd: PlayerCommand) {
        let _ = self.tx.send(cmd);
    }

    /// Por donde sale. Vacio = suma en el programa (y obedece al master); un
    /// nombre = su propia tarjeta, ajeno al programa.
    pub fn set_device(&self, name: &str) {
        let routing = if name.is_empty() {
            Routing::Program
        } else {
            Routing::Device(name.to_string())
        };
        self.send(PlayerCommand::SetRouting(routing));
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

    /// El volumen del reproductor **es el fader de su bus**: mueve la musica sin
    /// tocar los efectos, que es lo que hace falta para hablar encima. No pasa
    /// por el hilo — es un atomico, y aplicarlo debe ser inmediato aunque el
    /// motor este ocupado resolviendo la siguiente pista.
    pub fn set_volume(&self, volume: f32) {
        self.console
            .set_fader(BusId::Reproductor, volume.clamp(0.0, 1.5));
    }
    pub fn volume(&self) -> f32 {
        self.console.fader(BusId::Reproductor)
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
