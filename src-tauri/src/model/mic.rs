//! Preferencias globales de Mic; el estado activo nunca se persiste.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MicMode {
    #[default]
    Hold,
    Toggle,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MicConfig {
    pub mode: MicMode,
    pub main: bool,
    pub panel: bool,
    pub player: bool,
    pub level: f32,
    pub down_ms: u32,
    pub up_ms: u32,
    pub input: String,
    pub threshold_db: f32,
    pub detect_ms: u32,
    pub silence_ms: u32,
}

impl Default for MicConfig {
    fn default() -> Self {
        Self {
            mode: MicMode::Hold,
            main: true,
            panel: true,
            player: true,
            level: 0.2,
            down_ms: 200,
            up_ms: 500,
            input: String::new(),
            threshold_db: -30.0,
            detect_ms: 100,
            silence_ms: 1000,
        }
    }
}

impl MicConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(self.main || self.panel || self.player)
            || !self.level.is_finite()
            || !(0.0..=1.0).contains(&self.level)
            || !self.threshold_db.is_finite()
            || !(-80.0..=0.0).contains(&self.threshold_db)
            || self.down_ms > 5000
            || self.up_ms > 10000
            || self.detect_ms > 2000
            || !(50..=10000).contains(&self.silence_ms)
        {
            return Err("mic.invalid_config".into());
        }
        Ok(())
    }
}
