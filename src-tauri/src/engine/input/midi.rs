#[cfg(not(target_os = "android"))]
use super::midi_backend;
use super::midi_dispatch;
use super::midi_message::MidiInputEvent;
use super::midi_ports::{device_view, MidiInputDevice, MidiPortInfo};
#[cfg(not(target_os = "android"))]
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use tauri::Manager;

pub struct MidiEngine {
    tx: Mutex<Option<Sender<MidiRuntimeMsg>>>,
    capture: Arc<Mutex<Option<CaptureWait>>>,
    capture_id: AtomicU64,
    connected: Arc<Mutex<Vec<String>>>,
    #[cfg(target_os = "android")]
    ports: Mutex<Vec<MidiPortInfo>>,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    pub(crate) ble: super::midi_ble::BleMidi,
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
            #[cfg(target_os = "android")]
            ports: Mutex::new(Vec::new()),
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            ble: Default::default(),
        }
    }

    pub fn start(&self, app: AppHandle) {
        let (tx, rx) = mpsc::channel();
        *self.tx.lock().unwrap() = Some(tx.clone());
        let capture = Arc::clone(&self.capture);
        #[cfg(not(target_os = "android"))]
        let connected = Arc::clone(&self.connected);
        let _ = std::thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || {
                runtime_loop(
                    app,
                    rx,
                    tx,
                    capture,
                    #[cfg(not(target_os = "android"))]
                    connected,
                )
            });
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
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let connected = [connected, self.ble.connected()].concat();
        device_view(config, &connected, &self.available_ports())
    }
    #[cfg(target_os = "android")]
    pub(super) fn replace_android_devices(&self, ports: Vec<MidiPortInfo>, connected: Vec<String>) {
        *self.ports.lock().unwrap() = ports;
        *self.connected.lock().unwrap() = connected;
    }

    pub fn available_ports(&self) -> Vec<MidiPortInfo> {
        #[cfg(target_os = "android")]
        return self.ports.lock().unwrap().clone();
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        return [super::midi_ports::available_ports(), self.ble.ports()].concat();
        #[cfg(not(any(target_os = "android", target_os = "windows", target_os = "linux")))]
        super::midi_ports::available_ports()
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
    #[cfg(not(target_os = "android"))] connected: Arc<Mutex<Vec<String>>>,
) {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let sync_ble = {
        let state = app.state::<crate::core::AppState>();
        state
            .midi
            .ble
            .restore(&app, &state.config.lock().unwrap().midi);
        let tx = tx.clone();
        move |app: &AppHandle| {
            let state = app.state::<crate::core::AppState>();
            state
                .midi
                .ble
                .select(app, &state.config.lock().unwrap().midi, &tx);
        }
    };
    #[cfg(not(target_os = "android"))]
    let mut reconcile = {
        let mut connections = HashMap::new();
        move |app: &AppHandle| midi_backend::reconcile(app, &tx, &mut connections, &connected)
    };
    #[cfg(target_os = "android")]
    let reconcile = super::midi_android::attach(&app, tx);
    reconcile(&app);
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(MidiRuntimeMsg::Sync) => {
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                sync_ble(&app);
                reconcile(&app);
            }
            Ok(MidiRuntimeMsg::Incoming(event)) => handle_event(&app, &capture, event),
            Err(RecvTimeoutError::Timeout) => {
                #[cfg(not(target_os = "android"))]
                reconcile(&app);
            }
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
