/// Módulo: types.rs
/// Propósito: Esquema de datos completo. Todos los structs serializables que
/// cruzan la frontera IPC o se persisten en disco están aquí y sólo aquí.

use serde::{Deserialize, Serialize};

// ─── Botón ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonData {
    pub id:         String,
    pub index:      u32,
    pub label:      String,
    #[serde(default = "default_type", rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub path:       String,
    #[serde(default)]
    pub folder:     String,
    #[serde(default)]
    pub name:       String,
    pub color_bg:   String,
    pub color_text: String,
    #[serde(default = "default_vol")]
    pub vol:        f32,
    #[serde(default)]
    pub duration:   f64,
    #[serde(default)]
    pub duration_str: String,
    #[serde(default)]
    pub loop_mode:  bool,
    #[serde(default)]
    pub stop_other: bool,
    #[serde(default)]
    pub overlap:    bool,
    #[serde(default)]
    pub restart:    bool,
    #[serde(default)]
    pub shortcut:   String,
}

fn default_type() -> String { "audio".to_string() }
fn default_vol()  -> f32    { 1.0 }

// ─── Pestaña (Paleta) ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaletaData {
    pub id:       String,
    pub nombre:   String,
    #[serde(default = "default_rows")]
    pub rows:     u32,
    #[serde(default = "default_cols")]
    pub cols:     u32,
    #[serde(default)]
    pub audio_out: String,
    #[serde(default)]
    pub shortcut:  String,
    #[serde(default)]
    pub tab_bg:    String,
    #[serde(default)]
    pub tab_text:  String,
    pub botones:   Vec<ButtonData>,
}

fn default_rows() -> u32 { 5 }
fn default_cols() -> u32 { 5 }

// ─── Audio config ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    #[serde(default = "default_device")]
    pub out_main:    String,
    #[serde(default)]
    pub out_pre:     String,
    #[serde(default)]
    pub global_keys: bool,
    /// Atajos globales: detener todo, pestaña siguiente y anterior.
    #[serde(default)]
    pub key_stop:    String,
    #[serde(default)]
    pub key_next:    String,
    #[serde(default)]
    pub key_prev:    String,
    /// Modo de reproducción global del perfil: "normal" | "loop" | "overlap" | "restart" | "stop_others"
    #[serde(default = "default_playback_mode")]
    pub playback_mode: String,
}

fn default_device()        -> String { "default".to_string() }
fn default_playback_mode() -> String { "normal".to_string() }

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            out_main: default_device(), out_pre: String::new(), global_keys: false,
            key_stop: String::new(), key_next: String::new(), key_prev: String::new(),
            playback_mode: default_playback_mode(),
        }
    }
}

// ─── Locuciones Dinámicas (Fase 6) ────────────────────────────────────────────

/// Configuración del módulo opcional de locuciones. Dividido en dos bloques
/// independientes (hora / clima) para ahorrar recursos: el reloj no necesita
/// red y el clima solo consulta la API cuando su bloque está activo.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LocutionConfig {
    #[serde(default)]
    pub time_enabled:    bool,
    #[serde(default)]
    pub time_folder:     String,
    #[serde(default)]
    pub weather_enabled: bool,
    #[serde(default)]
    pub temp_folder:     String,
    #[serde(default)]
    pub hum_folder:      String,
    #[serde(default)]
    pub weather_city:    String,
    #[serde(default)]
    pub weather_lat:     f64,
    #[serde(default)]
    pub weather_lon:     f64,
    /// "metric" (°C) o "imperial" (°F), igual que el LF Automatizador.
    #[serde(default = "default_unit")]
    pub weather_unit:    String,
}

fn default_unit() -> String { "metric".to_string() }

// ─── Perfil ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileData {
    pub id:               String,
    pub name:             String,
    #[serde(default)]
    pub bg:               String,
    #[serde(default)]
    pub text:             String,
    #[serde(default)]
    pub audio:            AudioConfig,
    #[serde(default)]
    pub active_paleta_id: String,
    pub paletas:          Vec<PaletaData>,
}

// ─── Configuración global ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_true")]
    pub is_first_boot:          bool,
    #[serde(default)]
    pub weather_module_enabled: bool,
    #[serde(default)]
    pub lf_automatizador_link:  bool,
    #[serde(default = "default_theme")]
    pub theme:                  String,
    #[serde(default = "default_lang")]
    pub language:               String,
    #[serde(default)]
    pub active_profile_id:      String,
    /// true = formato 24 h (HH:MM:SS), false = 12 h sin AM/PM (hh:MM:SS).
    #[serde(default = "default_true")]
    pub clock_24h:              bool,
    #[serde(default)]
    pub locutions:              LocutionConfig,
    #[serde(default)]
    pub profiles:               Vec<ProfileData>,
}

fn default_true()  -> bool   { true }
fn default_theme() -> String { "dark".to_string() }
fn default_lang()  -> String { "es".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        let paleta = PaletaData {
            id: "paleta_1".to_string(), nombre: "Principal".to_string(),
            rows: 5, cols: 5,
            audio_out: String::new(), shortcut: String::new(),
            tab_bg: String::new(), tab_text: String::new(),
            botones: Vec::new(),
        };
        let profile = ProfileData {
            id: "perfil_1".to_string(), name: "Perfil 1".to_string(),
            bg: String::new(), text: String::new(),
            audio: AudioConfig::default(),
            active_paleta_id: "paleta_1".to_string(),
            paletas: vec![paleta],
        };
        Self {
            is_first_boot: true, weather_module_enabled: false,
            lf_automatizador_link: false, theme: "dark".to_string(),
            language: "es".to_string(),
            active_profile_id: "perfil_1".to_string(),
            clock_24h: true,
            locutions: LocutionConfig::default(),
            profiles: vec![profile],
        }
    }
}

/// Forma plana compatible con el frontend actual (get_grid_state).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GridState {
    pub columns: u32,
    pub rows:    u32,
    pub buttons: Vec<ButtonData>,
}
