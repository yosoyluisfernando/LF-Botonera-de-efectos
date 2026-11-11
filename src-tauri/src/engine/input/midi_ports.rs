use crate::model::{MidiConfig, MidiDeviceRef};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MidiInputDevice {
    pub id: String,
    pub name: String,
    pub label: String,
    pub selected: bool,
    pub available: bool,
    pub connected: bool,
    pub connection_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(target_os = "android", derive(serde::Deserialize))]
pub struct MidiPortInfo {
    pub id: String,
    pub name: String,
    #[cfg(target_os = "android")]
    #[serde(default)]
    pub connection_state: String,
    #[cfg(not(target_os = "android"))]
    pub index: u32,
    #[cfg(target_os = "linux")]
    pub native_id: String,
}

#[cfg(target_os = "windows")]
#[path = "midi_ports_windows.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "midi_ports_linux.rs"]
mod platform;

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn available_ports() -> Vec<MidiPortInfo> {
    platform::available_ports()
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android")))]
pub fn available_ports() -> Vec<MidiPortInfo> {
    Vec::new()
}

pub fn device_view(
    config: &MidiConfig,
    connected: &[String],
    ports: &[MidiPortInfo],
) -> Vec<MidiInputDevice> {
    let selected = config
        .inputs
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();
    let live = connected.iter().cloned().collect::<HashSet<_>>();
    let mut devices = ports
        .iter()
        .enumerate()
        .map(|(index, port)| {
            let previous = config.inputs.iter().find(|saved| saved.id == port.id);
            let name = preferred_name(
                &port.id,
                &port.name,
                previous.map(|saved| saved.name.as_str()),
            );
            let connected = live.contains(&port.id);
            MidiInputDevice {
                id: port.id.clone(),
                label: format!("{} ({})", name, index + 1),
                name,
                selected: selected.contains(&port.id),
                available: true,
                connected,
                connection_state: connection_state(port, connected).into(),
            }
        })
        .collect::<Vec<_>>();
    for saved in &config.inputs {
        if !saved.is_empty() && !devices.iter().any(|device| device.id == saved.id) {
            devices.push(MidiInputDevice {
                id: saved.id.clone(),
                name: saved.name.clone(),
                label: saved.name.clone(),
                selected: true,
                available: false,
                connected: false,
                connection_state: "disconnected".into(),
            });
        }
    }
    devices
}

pub fn selected_refs(ids: &[String], ports: &[MidiPortInfo]) -> Vec<MidiDeviceRef> {
    ids.iter()
        .map(|id| {
            let name = ports
                .iter()
                .find(|port| &port.id == id)
                .map(|port| port.name.clone())
                .unwrap_or_else(|| id.clone());
            MidiDeviceRef {
                id: id.clone(),
                name,
            }
        })
        .collect()
}

fn connection_state(_port: &MidiPortInfo, connected: bool) -> &str {
    if connected {
        return "connected";
    }
    #[cfg(target_os = "android")]
    if matches!(_port.connection_state.as_str(), "connecting" | "failed") {
        return &_port.connection_state;
    }
    "disconnected"
}

pub fn preferred_name(id: &str, candidate: &str, previous: Option<&str>) -> String {
    let address = id
        .strip_prefix("ble:")
        .or_else(|| id.strip_prefix("ble-desktop:"));
    let generic = |name: &str| name.trim().is_empty() || name == id || Some(name) == address;
    if generic(candidate) {
        if let Some(previous) = previous.filter(|name| !generic(name)) {
            return previous.into();
        }
    }
    if candidate.trim().is_empty() {
        id.into()
    } else {
        candidate.into()
    }
}

#[cfg(test)]
#[path = "midi_ports_tests.rs"]
mod tests;
