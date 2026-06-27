/// Módulo: types_preload.rs
/// Propósito: esquema serializable de la configuración de PRECARGA de audio.
/// TODA la lógica (caché, decodificación, expulsión, validación, valores por
/// defecto, conversión de unidades) vive en Rust; la UI solo elige opciones y
/// las envía (Regla 4). La caché/decodificación llegan en etapas posteriores
/// (ver Documentación/PLAN_PRECARGA.md). Aquí solo viven los datos y sus reglas.
use serde::{Deserialize, Serialize};

/// Estrategia de llenado de la caché de precarga.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreloadStrategy {
    FullProfile,
    VisibleTabs,
    OnPlay,
}

impl Default for PreloadStrategy {
    fn default() -> Self {
        PreloadStrategy::OnPlay
    }
}

impl PreloadStrategy {
    /// Parsea la cadena que envía la UI a la estrategia (validación en Rust).
    pub fn from_str(s: &str) -> Self {
        match s {
            "full_profile" => PreloadStrategy::FullProfile,
            "visible_tabs" => PreloadStrategy::VisibleTabs,
            _ => PreloadStrategy::OnPlay,
        }
    }

    /// Cadena estable para la UI.
    pub fn as_str(&self) -> &'static str {
        match self {
            PreloadStrategy::FullProfile => "full_profile",
            PreloadStrategy::VisibleTabs => "visible_tabs",
            PreloadStrategy::OnPlay => "on_play",
        }
    }
}

/// Configuración de precarga, persistida dentro de AppConfig (ajuste global).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloadConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ram_mb")]
    pub ram_budget_mb: u32,
    #[serde(default = "default_max_dur")]
    pub max_duration_s: u32,
    #[serde(default)]
    pub strategy: PreloadStrategy,
    /// Unidad canónica del TTL = horas (la UI puede pedir días; Rust convierte).
    #[serde(default = "default_evict_hours")]
    pub evict_after_hours: u32,
    /// ¿Ya se preguntó al usuario tras la actualización?
    #[serde(default)]
    pub prompted: bool,
}

fn default_ram_mb() -> u32 {
    128
}
fn default_max_dur() -> u32 {
    10
}
fn default_evict_hours() -> u32 {
    72
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ram_budget_mb: default_ram_mb(),
            max_duration_s: default_max_dur(),
            strategy: PreloadStrategy::default(),
            evict_after_hours: default_evict_hours(),
            prompted: false,
        }
    }
}
