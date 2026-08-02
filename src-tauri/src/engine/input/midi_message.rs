use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MidiInputEvent {
    pub device_id: String,
    pub device_name: String,
    pub message: String,
    pub channel: u8,
    pub number: u8,
    pub value: u8,
}

impl MidiInputEvent {
    pub fn display(&self) -> String {
        format!(
            "{} ch {} {} {}",
            self.device_name,
            self.channel.saturating_add(1),
            label(&self.message),
            self.number
        )
    }
}

pub fn parse(device_id: &str, device_name: &str, data: &[u8]) -> Option<MidiInputEvent> {
    let status = *data.first()?;
    let channel = status & 0x0f;
    match status & 0xf0 {
        0x90 if data.len() >= 3 && data[2] > 0 => {
            event(device_id, device_name, "note", channel, data[1], data[2])
        }
        0xb0 if data.len() >= 3 && data[2] > 0 => {
            event(device_id, device_name, "cc", channel, data[1], data[2])
        }
        0xc0 if data.len() >= 2 => event(device_id, device_name, "program", channel, data[1], 1),
        _ => None,
    }
}

fn event(
    device_id: &str,
    device_name: &str,
    message: &str,
    channel: u8,
    number: u8,
    value: u8,
) -> Option<MidiInputEvent> {
    Some(MidiInputEvent {
        device_id: device_id.to_string(),
        device_name: device_name.to_string(),
        message: message.to_string(),
        channel,
        number,
        value,
    })
}

fn label(message: &str) -> &str {
    match message {
        "note" => "nota",
        "cc" => "CC",
        "program" => "programa",
        _ => "MIDI",
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn note_on_zero_velocity_is_ignored() {
        assert!(parse("a", "dev", &[0x90, 36, 0]).is_none());
    }

    #[test]
    fn parses_supported_trigger_messages() {
        let note = parse("a", "dev", &[0x91, 36, 127]).unwrap();
        assert_eq!(
            (note.message.as_str(), note.channel, note.number),
            ("note", 1, 36)
        );
        let cc = parse("a", "dev", &[0xb0, 10, 64]).unwrap();
        assert_eq!((cc.message.as_str(), cc.channel, cc.number), ("cc", 0, 10));
        let pc = parse("a", "dev", &[0xc2, 5]).unwrap();
        assert_eq!(
            (pc.message.as_str(), pc.channel, pc.number),
            ("program", 2, 5)
        );
    }
}
