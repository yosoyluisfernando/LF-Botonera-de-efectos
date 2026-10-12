//! Control de Mic. El hilo de captura posee el stream, nunca el frontend.
pub mod capture;
mod worker;
use crate::{
    engine::console::ConsoleEngine,
    model::{mic::MicMode, AppConfig},
};
use serde::Serialize;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

pub enum Command {
    Preview(Option<String>),
    Input(String, bool),
    ReleaseLocal,
    Reset,
    Shutdown,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub mode: MicMode,
    pub active: bool,
    pub listening: bool,
    pub level_db: f32,
    pub error: Option<String>,
    pub preview_level_db: f32,
    pub preview_error: Option<String>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            mode: MicMode::Hold,
            active: false,
            listening: false,
            level_db: -100.0,
            error: None,
            preview_level_db: -100.0,
            preview_error: None,
        }
    }
}

pub struct MicEngine {
    tx: Sender<Command>,
    rx: Mutex<Option<Receiver<Command>>>,
    snapshot: Arc<Mutex<Snapshot>>,
}
impl MicEngine {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
            snapshot: Arc::new(Mutex::new(Snapshot::default())),
        }
    }
    pub fn start(
        &self,
        app: tauri::AppHandle,
        console: Arc<ConsoleEngine>,
        config: Arc<Mutex<AppConfig>>,
    ) {
        if let Some(rx) = self.rx.lock().unwrap().take() {
            let snapshot = Arc::clone(&self.snapshot);
            std::thread::spawn(move || worker::run(rx, console, config, snapshot, app));
        }
    }
    pub fn send(&self, command: Command) -> Result<(), String> {
        self.tx.send(command).map_err(|_| "mic.unavailable".into())
    }
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().unwrap().clone()
    }
    pub fn sender(&self) -> Sender<Command> {
        self.tx.clone()
    }
}
