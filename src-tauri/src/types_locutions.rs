/// Módulo: types_locutions.rs
/// Propósito: Tipos persistidos del módulo opcional de locuciones dinámicas.

use serde::{Deserialize, Serialize};

/// Configuración de locuciones de hora y clima.
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
