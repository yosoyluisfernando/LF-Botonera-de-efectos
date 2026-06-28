/// Módulo: types_norm.rs
/// Propósito: configuración global del normalizador de pistas.
use serde::{Deserialize, Serialize};

/// Parámetros globales del normalizador automático.
/// Modificable desde el engranaje del editor de pistas.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NormConfig {
    /// "lufs" → normaliza por loudness integrado (estándar streaming)
    /// "peak" → normaliza al pico máximo sin referencia de loudness
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Objetivo en el modo activo:
    /// LUFS → ej. -14.0; Peak → ej. -1.0
    #[serde(default = "default_target")]
    pub target: f64,
    /// Techo de pico (solo aplicable en modo LUFS): ninguna muestra superará
    /// este valor dBFS después de aplicar la ganancia sugerida.
    #[serde(default = "default_ceiling")]
    pub ceiling_db: f64,
}

fn default_mode() -> String {
    "lufs".to_string()
}
fn default_target() -> f64 {
    -14.0
}
fn default_ceiling() -> f64 {
    -1.0
}

impl Default for NormConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            target: default_target(),
            ceiling_db: default_ceiling(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_lufs_minus14() {
        let c = NormConfig::default();
        assert_eq!(c.mode, "lufs");
        assert_eq!(c.target, -14.0);
        assert_eq!(c.ceiling_db, -1.0);
    }
}
