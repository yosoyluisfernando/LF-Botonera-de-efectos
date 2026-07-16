use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LfaButton {
    pub id: u32,
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_type", rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub folder: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub bg: String,
    #[serde(default = "default_text")]
    pub text: String,
    #[serde(default = "default_vol")]
    pub vol: f32,
    #[serde(default, rename = "loop")]
    pub loop_mode: bool,
    #[serde(default, rename = "stopOther")]
    pub stop_other: bool,
    #[serde(default)]
    pub overlap: bool,
    #[serde(default)]
    pub restart: bool,
    #[serde(default)]
    pub shortcut: String,
}

pub(super) fn default_type() -> String {
    "audio".to_string()
}
pub(super) fn default_text() -> String {
    "#FFFFFF".to_string()
}
pub(super) fn default_vol() -> f32 {
    1.0
}

#[derive(Serialize, Deserialize)]
pub struct LfaPaleta {
    pub nombre: String,
    #[serde(default = "default_dim")]
    pub rows: u32,
    #[serde(default = "default_dim")]
    pub cols: u32,
    #[serde(default = "default_out", rename = "audioOut")]
    pub audio_out: String,
    #[serde(default)]
    pub shortcut: String,
    #[serde(default, rename = "tabBg")]
    pub tab_bg: String,
    #[serde(default, rename = "tabText")]
    pub tab_text: String,
    pub botones: Vec<LfaButton>,
}

pub(super) fn default_dim() -> u32 {
    5
}
pub(super) fn default_out() -> String {
    "global".to_string()
}

#[derive(Serialize, Deserialize, Default)]
pub struct LfaKeys {
    #[serde(default, rename = "stopAll")]
    pub stop_all: String,
    #[serde(default)]
    pub next: String,
    #[serde(default)]
    pub prev: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LfaConfig {
    #[serde(default, rename = "outMain")]
    pub out_main: String,
    #[serde(default, rename = "outPre")]
    pub out_pre: String,
    #[serde(default)]
    pub keys: LfaKeys,
}

#[derive(Serialize, Deserialize)]
pub struct LfaProfile {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub bg: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub config: LfaConfig,
    pub paletas: Vec<LfaPaleta>,
    #[serde(
        default,
        rename = "fixedButtons",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub fixed_buttons: Vec<crate::model::ButtonData>,
}
