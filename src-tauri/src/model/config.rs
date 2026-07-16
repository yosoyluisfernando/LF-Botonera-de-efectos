//! Modulo: model/config.rs
//! Proposito: `AppConfig`, la raiz de la configuracion que se persiste en
//! `botonera_config.json`. Son las PREFERENCIAS de la aplicacion; el contenido
//! que crea el usuario (perfil → paleta → boton) vive en `model/content.rs`.
//! Cada bloque de ajustes tiene su propio modulo y aqui solo se compone.
use crate::model::console::ConsoleConfig;
use crate::model::content::{PaletaData, ProfileData};
use crate::model::fade::FadeConfig;
use crate::model::fixed_panel::FixedPanelConfig;
use crate::model::locutions::LocutionConfig;
use crate::model::norm::{CueDetectConfig, NormConfig};
use crate::model::playback::PlaybackProgressConfig;
use crate::model::player::PlayerConfig;
use crate::model::preload::PreloadConfig;
use crate::model::startup::StartupPromptState;
use crate::model::waveform_cache::WaveformCacheConfig;
use serde::{Deserialize, Serialize};
// AudioConfig vive en su propio módulo; se re-exporta para que el resto del
// código siga usándolo como `crate::model::AudioConfig` sin cambios.
pub use crate::model::audio::AudioConfig;

/// Todo campo nuevo lleva `#[serde(default)]`: una configuracion guardada por
/// una version anterior debe seguir cargando (regla 6).
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
    /// Cómo abre la consola de audio: "window" (flotante) | "modal".
    #[serde(default = "default_console_mode")]
    pub console_mode: String,
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
    pub fixed_panel: FixedPanelConfig,
    /// Reproductor auxiliar: uno solo y global (no por perfil).
    #[serde(default)]
    pub player: PlayerConfig,
    /// Los faders de la consola que no viven en otro sitio. El máster está en
    /// `AudioConfig` y el del reproductor en `PlayerConfig`.
    #[serde(default)]
    pub console: ConsoleConfig,
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
/// La consola nace en ventana flotante: es lo que se recomienda mientras esté
/// en pruebas, porque deja ver la botonera y la consola a la vez.
fn default_console_mode() -> String {
    "window".to_string()
}

fn default_editor_mode() -> String {
    "modal".to_string()
}

/// Configuracion de fabrica: un perfil con una paleta de 5x5 vacia.
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
            fixed_buttons: Vec::new(),
        };
        Self {
            is_first_boot: true,
            weather_module_enabled: false,
            lf_automatizador_link: false,
            theme: default_theme(),
            language: default_lang(),
            button_text_size: default_button_text_size(),
            editor_mode: default_editor_mode(),
            console_mode: default_console_mode(),
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
            fixed_panel: FixedPanelConfig::default(),
            player: PlayerConfig::default(),
            console: ConsoleConfig::default(),
            profiles: vec![profile],
        }
    }
}
