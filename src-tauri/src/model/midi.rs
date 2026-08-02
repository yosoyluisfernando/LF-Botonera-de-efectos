use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct MidiDeviceRef {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}

impl MidiDeviceRef {
    pub fn is_empty(&self) -> bool {
        self.id.trim().is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct MidiBinding {
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub device_name: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub channel: u8,
    #[serde(default)]
    pub number: u8,
}

impl MidiBinding {
    pub fn is_empty(&self) -> bool {
        self.device_id.trim().is_empty() || self.message.trim().is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct MidiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub inputs: Vec<MidiDeviceRef>,
}

impl MidiConfig {
    pub fn is_default(&self) -> bool {
        !self.enabled && self.inputs.is_empty()
    }
}
