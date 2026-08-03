use super::MidiPortInfo;
use midir::MidiInput;
use std::collections::HashMap;

const APP_CLIENT_NAME: &str = "LF Botonera de Efectos";

pub fn available_ports() -> Vec<MidiPortInfo> {
    let Ok(input) = MidiInput::new(APP_CLIENT_NAME) else {
        return Vec::new();
    };
    let mut occurrences = HashMap::<String, u32>::new();
    input
        .ports()
        .into_iter()
        .enumerate()
        .filter_map(|(index, port)| {
            let native_id = port.id();
            let name = stable_name(&input.port_name(&port).ok()?, &native_id);
            let occurrence = occurrences.entry(name.clone()).or_default();
            let persisted_id = persisted_id(&name, *occurrence);
            *occurrence += 1;
            Some(MidiPortInfo {
                id: persisted_id,
                name,
                index: index as u32,
                native_id,
            })
        })
        .collect()
}

fn stable_name(name: &str, native_id: &str) -> String {
    name.strip_suffix(&format!(" {native_id}"))
        .unwrap_or(name)
        .to_string()
}

fn persisted_id(name: &str, occurrence: u32) -> String {
    let encoded_name = name
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("alsa:{encoded_name}:{occurrence}")
}

#[cfg(test)]
mod tests {
    use super::{persisted_id, stable_name};

    #[test]
    fn removes_volatile_alsa_address_from_name() {
        assert_eq!(
            stable_name("Controller:MIDI 128:0", "128:0"),
            "Controller:MIDI"
        );
    }

    #[test]
    fn persisted_id_uses_stable_name_and_duplicate_position() {
        assert_eq!(persisted_id("MIDI", 1), "alsa:4d494449:1");
    }
}
