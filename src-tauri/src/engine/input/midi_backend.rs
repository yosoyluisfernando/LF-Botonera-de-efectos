use super::midi::MidiRuntimeMsg;
use super::midi_ports::MidiPortInfo;
use std::sync::mpsc::Sender;

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

pub(super) use platform::{connect_port, MidiConnection};
