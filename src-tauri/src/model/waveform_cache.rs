/// Configuracion de la cache persistente de waveform del editor.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheConfig {
    #[serde(default = "default_max_mb")]
    pub max_mb: u32,
    #[serde(default = "default_max_age_days")]
    pub max_age_days: u32,
}

fn default_max_mb() -> u32 {
    100
}

fn default_max_age_days() -> u32 {
    30
}

impl WaveformCacheConfig {
    pub fn sanitized(&self) -> Self {
        Self {
            max_mb: self.max_mb.clamp(1, 10_000),
            max_age_days: self.max_age_days.clamp(1, 31),
        }
    }
}

impl Default for WaveformCacheConfig {
    fn default() -> Self {
        Self {
            max_mb: default_max_mb(),
            max_age_days: default_max_age_days(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_100mb_and_30_days() {
        let cfg = WaveformCacheConfig::default();
        assert_eq!(cfg.max_mb, 100);
        assert_eq!(cfg.max_age_days, 30);
    }

    #[test]
    fn clamps_days_to_31() {
        let cfg = WaveformCacheConfig {
            max_mb: 0,
            max_age_days: 99,
        }
        .sanitized();
        assert_eq!(cfg.max_mb, 1);
        assert_eq!(cfg.max_age_days, 31);
    }
}
