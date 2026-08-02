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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiPortInfo {
    pub id: String,
    pub name: String,
    pub index: u32,
}

#[cfg(target_os = "windows")]
#[path = "midi_ports_windows.rs"]
mod platform;

#[cfg(target_os = "windows")]
pub fn available_ports() -> Vec<MidiPortInfo> {
    platform::available_ports()
}

#[cfg(not(target_os = "windows"))]
pub fn available_ports() -> Vec<MidiPortInfo> {
    Vec::new()
}

pub fn device_view(config: &MidiConfig, connected: &[String]) -> Vec<MidiInputDevice> {
    let ports = available_ports();
    let selected = config
        .inputs
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();
    let live = connected.iter().cloned().collect::<HashSet<_>>();
    let mut devices = ports
        .iter()
        .enumerate()
        .map(|(index, port)| MidiInputDevice {
            id: port.id.clone(),
            name: port.name.clone(),
            label: format!("{} ({})", port.name, index + 1),
            selected: selected.contains(&port.id),
            available: true,
            connected: live.contains(&port.id),
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
            });
        }
    }
    devices
}

pub fn selected_refs(ids: &[String]) -> Vec<MidiDeviceRef> {
    let ports = available_ports();
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
