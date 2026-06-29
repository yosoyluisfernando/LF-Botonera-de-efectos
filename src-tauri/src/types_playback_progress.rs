/// Modulo: types_playback_progress.rs
/// Proposito: configuracion global de la barra de progreso de reproduccion.
use serde::{Deserialize, Serialize};

const DEFAULT_SEEK_STEP_S: u32 = 10;
const ALLOWED_STEPS: [u32; 6] = [1, 2, 5, 10, 20, 30];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaybackProgressConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_seek_step_s")]
    pub seek_step_s: u32,
}

fn default_seek_step_s() -> u32 {
    DEFAULT_SEEK_STEP_S
}

impl PlaybackProgressConfig {
    pub fn sanitized(mut self) -> Self {
        if !ALLOWED_STEPS.contains(&self.seek_step_s) {
            self.seek_step_s = DEFAULT_SEEK_STEP_S;
        }
        self
    }
}

impl Default for PlaybackProgressConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            seek_step_s: DEFAULT_SEEK_STEP_S,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_step_falls_back_to_default() {
        let cfg = PlaybackProgressConfig {
            enabled: true,
            seek_step_s: 7,
        }
        .sanitized();
        assert_eq!(cfg.seek_step_s, DEFAULT_SEEK_STEP_S);
    }
}
