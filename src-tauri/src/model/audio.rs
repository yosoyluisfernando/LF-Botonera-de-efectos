/// Módulo: types_audio.rs
/// Propósito: configuración de audio por perfil (salidas, atajos globales, modo
/// de reproducción y volumen master). Parte del esquema serializable; separado
/// de types.rs por responsabilidad única. Se re-exporta desde types.rs.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    #[serde(default = "default_device")]
    pub out_main: String,
    #[serde(default)]
    pub out_pre: String,
    #[serde(default)]
    pub global_keys: bool,
    #[serde(default)]
    pub key_stop: String,
    #[serde(default)]
    pub key_next: String,
    #[serde(default)]
    pub key_prev: String,
    #[serde(default = "default_playback_mode")]
    pub playback_mode: String,
    #[serde(default)]
    pub solo_mode: bool,
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    #[serde(default)]
    pub master_volume_remember: bool,
    #[serde(default)]
    pub master_volume_boost: bool,
}

fn default_device() -> String {
    "default".to_string()
}
fn default_playback_mode() -> String {
    "normal".to_string()
}
fn default_master_volume() -> f32 {
    1.0
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            out_main: default_device(),
            out_pre: String::new(),
            global_keys: false,
            key_stop: String::new(),
            key_next: String::new(),
            key_prev: String::new(),
            playback_mode: default_playback_mode(),
            solo_mode: false,
            master_volume: default_master_volume(),
            master_volume_remember: false,
            master_volume_boost: false,
        }
    }
}
