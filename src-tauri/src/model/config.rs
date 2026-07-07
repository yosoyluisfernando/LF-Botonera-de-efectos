/// Módulo: types.rs
/// Propósito: Esquema de datos serializable compartido entre Rust, UI y disco.
use crate::model::fade::FadeConfig;
use crate::model::locutions::LocutionConfig;
use crate::model::norm::{CueDetectConfig, NormConfig};
use crate::model::playback::PlaybackProgressConfig;
use crate::model::preload::PreloadConfig;
use crate::model::startup::StartupPromptState;
use crate::model::waveform_cache::WaveformCacheConfig;
use serde::{Deserialize, Serialize};

// AudioConfig vive en su propio módulo; se re-exporta para que el resto del
// código siga usándolo como `crate::model::AudioConfig` sin cambios.
pub use crate::model::audio::AudioConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonData {
    pub id: String,
    pub index: u32,
    pub label: String,
    #[serde(default = "default_type", rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub folder: String,
    #[serde(default)]
    pub name: String,
    pub color_bg: String,
    pub color_text: String,
    #[serde(default = "default_vol")]
    pub vol: f32,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub duration_str: String,
    #[serde(default)]
    pub loop_mode: bool,
    #[serde(default)]
    pub stop_other: bool,
    #[serde(default)]
    pub overlap: bool,
    #[serde(default)]
    pub restart: bool,
    #[serde(default)]
    pub shortcut: String,
}

fn default_type() -> String { "audio".to_string() }
fn default_vol() -> f32 { 1.0 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaletaData {
    pub id: String,
    pub nombre: String,
    #[serde(default = "default_rows")]
    pub rows: u32,
    #[serde(default = "default_cols")]
    pub cols: u32,
    #[serde(default)]
    pub audio_out: String,
    #[serde(default)]
    pub shortcut: String,
    #[serde(default)]
    pub tab_bg: String,
    #[serde(default)]
    pub tab_text: String,
    pub botones: Vec<ButtonData>,
}

fn default_rows() -> u32 { 5 }
fn default_cols() -> u32 { 5 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub bg: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub active_paleta_id: String,
    pub paletas: Vec<PaletaData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_true")]
    pub is_first_boot: bool,
    #[serde(default)]
    pub weather_module_enabled: bool,
    #[serde(default)]
    pub lf_automatizador_link: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_lang")]
    pub language: String,
    #[serde(default = "default_button_text_size")]
    pub button_text_size: String,
    #[serde(default = "default_editor_mode")]
    pub editor_mode: String,
    #[serde(default)]
    pub active_profile_id: String,
    #[serde(default = "default_true")]
    pub clock_24h: bool,
    #[serde(default)]
    pub last_update_check: i64,
    #[serde(default)]
    pub locutions: LocutionConfig,
    #[serde(default)]
    pub preload: PreloadConfig,
    #[serde(default)]
    pub norm: NormConfig,
    #[serde(default)]
    pub cue_detect: CueDetectConfig,
    #[serde(default)]
    pub norm_prompted: bool,
    #[serde(default)]
    pub fade: FadeConfig,
    #[serde(default)]
    pub playback_progress: PlaybackProgressConfig,
    #[serde(default)]
    pub waveform_cache: WaveformCacheConfig,
    #[serde(default)]
    pub startup: StartupPromptState,
    #[serde(default)]
    pub profiles: Vec<ProfileData>,
}

fn default_true() -> bool {
    true
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_lang() -> String {
    "es".to_string()
}
fn default_button_text_size() -> String {
    "normal".to_string()
}
fn default_editor_mode() -> String {
    "modal".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let paleta = PaletaData {
            id: "paleta_1".to_string(),
            nombre: "Principal".to_string(),
            rows: 5,
            cols: 5,
            audio_out: String::new(),
            shortcut: String::new(),
            tab_bg: String::new(),
            tab_text: String::new(),
            botones: Vec::new(),
        };
        let profile = ProfileData {
            id: "perfil_1".to_string(),
            name: "Perfil 1".to_string(),
            bg: String::new(),
            text: String::new(),
            audio: AudioConfig::default(),
            active_paleta_id: "paleta_1".to_string(),
            paletas: vec![paleta],
        };
        Self {
            is_first_boot: true,
            weather_module_enabled: false,
            lf_automatizador_link: false,
            theme: default_theme(),
            language: default_lang(),
            button_text_size: default_button_text_size(),
            editor_mode: default_editor_mode(),
            active_profile_id: "perfil_1".to_string(),
            clock_24h: true,
            last_update_check: 0,
            locutions: LocutionConfig::default(),
            preload: PreloadConfig::default(),
            norm: NormConfig::default(),
            cue_detect: CueDetectConfig::default(),
            norm_prompted: false,
            fade: FadeConfig::default(),
            playback_progress: PlaybackProgressConfig::default(),
            waveform_cache: WaveformCacheConfig::default(),
            startup: StartupPromptState::default(),
            profiles: vec![profile],
        }
    }
}
