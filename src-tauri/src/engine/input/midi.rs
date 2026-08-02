use super::midi_backend::{connect_port, MidiConnection};
use super::midi_dispatch;
use super::midi_message::MidiInputEvent;
use super::midi_ports::{available_ports, device_view, MidiInputDevice};
use crate::core::AppState;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub struct MidiEngine {
    tx: Mutex<Option<Sender<MidiRuntimeMsg>>>,
    capture: Arc<Mutex<Option<CaptureWait>>>,
    capture_id: AtomicU64,
    connected: Arc<Mutex<Vec<String>>>,
}

struct CaptureWait {
    id: u64,
    tx: Sender<MidiInputEvent>,
}

pub struct MidiCaptureWait {
    id: u64,
    rx: Receiver<MidiInputEvent>,
    capture: Arc<Mutex<Option<CaptureWait>>>,
}

pub(super) enum MidiRuntimeMsg {
    Sync,
    Incoming(MidiInputEvent),
}

impl MidiEngine {
    pub fn new() -> Self {
        Self {
            tx: Mutex::new(None),
            capture: Arc::new(Mutex::new(None)),
            capture_id: AtomicU64::new(1),
            connected: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self, app: AppHandle) {
        let (tx, rx) = mpsc::channel();
        *self.tx.lock().unwrap() = Some(tx.clone());
        let capture = Arc::clone(&self.capture);
        let connected = Arc::clone(&self.connected);
        let _ = std::thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || runtime_loop(app, rx, tx, capture, connected));
    }

    pub fn sync(&self) {
        if let Some(tx) = self.tx.lock().unwrap().as_ref() {
            let _ = tx.send(MidiRuntimeMsg::Sync);
        }
    }

    pub fn begin_capture(&self) -> Result<MidiCaptureWait, String> {
        let (tx, rx) = mpsc::channel();
        let id = self.capture_id.fetch_add(1, Ordering::Relaxed);
        {
            let mut guard = self.capture.lock().unwrap();
            if guard.is_some() {
                return Err("midi_capture_busy".into());
            }
            *guard = Some(CaptureWait { id, tx });
        }
        Ok(MidiCaptureWait {
            id,
            rx,
            capture: Arc::clone(&self.capture),
        })
    }

    pub fn cancel_capture(&self) {
        self.capture.lock().unwrap().take();
    }

    pub fn devices(&self, config: &crate::model::MidiConfig) -> Vec<MidiInputDevice> {
        let connected = self.connected.lock().unwrap().clone();
        device_view(config, &connected)
    }
}

impl MidiCaptureWait {
    pub fn wait(self, timeout: Duration) -> Result<MidiInputEvent, String> {
        self.rx.recv_timeout(timeout).map_err(|error| match error {
            RecvTimeoutError::Timeout => "midi_capture_timeout".to_string(),
            RecvTimeoutError::Disconnected => "midi_capture_cancelled".to_string(),
        })
    }
}

impl Drop for MidiCaptureWait {
    fn drop(&mut self) {
        let mut guard = self.capture.lock().unwrap();
        if guard.as_ref().map(|wait| wait.id) == Some(self.id) {
            *guard = None;
        }
    }
}

fn runtime_loop(
    app: AppHandle,
    rx: mpsc::Receiver<MidiRuntimeMsg>,
    tx: Sender<MidiRuntimeMsg>,
    capture: Arc<Mutex<Option<CaptureWait>>>,
    connected: Arc<Mutex<Vec<String>>>,
) {
    let mut connections = HashMap::new();
    reconcile(&app, &tx, &mut connections, &connected);
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(MidiRuntimeMsg::Sync) => reconcile(&app, &tx, &mut connections, &connected),
            Ok(MidiRuntimeMsg::Incoming(event)) => handle_event(&app, &capture, event),
            Err(RecvTimeoutError::Timeout) => reconcile(&app, &tx, &mut connections, &connected),
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn handle_event(app: &AppHandle, capture: &Arc<Mutex<Option<CaptureWait>>>, event: MidiInputEvent) {
    if let Some(wait) = capture.lock().unwrap().take() {
        let _ = wait.tx.send(event);
        return;
    }
    if let Err(error) = midi_dispatch::dispatch(app, &event) {
        eprintln!("Error en MIDI: {error}");
    }
}

fn reconcile(
    app: &AppHandle,
    tx: &Sender<MidiRuntimeMsg>,
    connections: &mut HashMap<String, MidiConnection>,
    connected: &Arc<Mutex<Vec<String>>>,
) {
    let state = app.state::<AppState>();
    let config = state.config.lock().unwrap().midi.clone();
    let live = available_ports();
    let live_ids = live
        .iter()
        .map(|port| port.id.clone())
        .collect::<HashSet<_>>();
    let selected = config
        .inputs
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();
    connections.retain(|id, _| config.enabled && selected.contains(id) && live_ids.contains(id));
    if config.enabled {
        for port in live.iter().filter(|port| selected.contains(&port.id)) {
            if !connections.contains_key(&port.id) {
                if let Some(connection) = connect_port(port, tx.clone()) {
                    connections.insert(port.id.clone(), connection);
                }
            }
        }
    }
    publish_connected(app, connections, connected);
}

fn publish_connected(
    app: &AppHandle,
    connections: &HashMap<String, MidiConnection>,
    connected: &Arc<Mutex<Vec<String>>>,
) {
    let mut ids = connections.keys().cloned().collect::<Vec<_>>();
    ids.sort();
    let mut guard = connected.lock().unwrap();
    if *guard != ids {
        *guard = ids;
        let _ = app.emit("midi-devices-changed", ());
    }
}
