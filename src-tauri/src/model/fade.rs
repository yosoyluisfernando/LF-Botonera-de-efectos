/// Módulo: types_fade.rs
/// Propósito: configuración global de fundidos (fade in / fade out).
use serde::{Deserialize, Serialize};

/// Parámetros globales de fundidos.
/// Configurables desde los ajustes generales de la aplicación.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FadeConfig {
    /// Tiempo de fundido de entrada al iniciar la reproducción (segundos).
    #[serde(default = "default_zero")]
    pub fade_in_s: f64,
    /// Tiempo de fundido de salida al presionar Detener (segundos).
    #[serde(default = "default_zero")]
    pub fade_out_stop_s: f64,
    /// Tiempo de fundido de salida al terminar naturalmente el audio (segundos).
    #[serde(default = "default_zero")]
    pub fade_out_end_s: f64,
}

fn default_zero() -> f64 {
    0.0
}

impl Default for FadeConfig {
    fn default() -> Self {
        Self {
            fade_in_s: 0.0,
            fade_out_stop_s: 0.0,
            fade_out_end_s: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zeros() {
        let c = FadeConfig::default();
        assert_eq!(c.fade_in_s, 0.0);
        assert_eq!(c.fade_out_stop_s, 0.0);
        assert_eq!(c.fade_out_end_s, 0.0);
    }
}
