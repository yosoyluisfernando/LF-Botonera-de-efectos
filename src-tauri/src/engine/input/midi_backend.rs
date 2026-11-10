use super::midi::MidiRuntimeMsg;
use super::midi_ports::MidiPortInfo;
use crate::core::AppState;
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc::Sender, Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "windows")]
#[path = "midi_backend_windows.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "midi_backend_linux.rs"]
mod platform;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod platform {
    use super::*;

    pub struct MidiConnection;

    pub fn connect_port(
        _port: &MidiPortInfo,
        _tx: Sender<MidiRuntimeMsg>,
    ) -> Option<MidiConnection> {
        None
    }
}

use platform::{connect_port, MidiConnection};

pub(super) fn reconcile(
    app: &AppHandle,
    tx: &Sender<MidiRuntimeMsg>,
    connections: &mut HashMap<String, MidiConnection>,
    connected: &Arc<Mutex<Vec<String>>>,
) {
    let state = app.state::<AppState>();
    let config = state.config.lock().unwrap().midi.clone();
    let live = super::midi_ports::available_ports();
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
