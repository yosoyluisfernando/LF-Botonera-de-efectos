//! Estado de interacción y detector de nivel; sin UI ni dispositivos.
use crate::model::mic::{MicConfig, MicMode};
use std::collections::HashSet;

#[derive(Default)]
pub struct MicState {
    held: HashSet<String>,
    pub latched: bool,
    pub listening: bool,
    pub voice: bool,
    above_ms: u64,
    below_ms: u64,
}

impl MicState {
    pub fn input(&mut self, mode: MicMode, source: String, pressed: bool) {
        if !pressed {
            self.held.remove(&source);
        } else if self.held.insert(source) {
            match mode {
                MicMode::Hold => (),
                MicMode::Toggle => self.latched = !self.latched,
                MicMode::Auto => self.listening = !self.listening,
            }
        }
    }

    pub fn release_local(&mut self) {
        self.held.retain(|s| s.starts_with("global:"));
    }

    pub fn active(&self, mode: MicMode) -> bool {
        match mode {
            MicMode::Hold => !self.held.is_empty(),
            MicMode::Toggle => self.latched,
            MicMode::Auto => self.listening && self.voice,
        }
    }

    pub fn detect(&mut self, db: f32, elapsed_ms: u64, cfg: &MicConfig) {
        if !self.listening {
            self.clear_detector();
            return;
        }
        let threshold = cfg.threshold_db - if self.voice { 3.0 } else { 0.0 };
        if db >= threshold {
            self.below_ms = 0;
            self.above_ms = self.above_ms.saturating_add(elapsed_ms);
            if self.above_ms >= cfg.detect_ms as u64 {
                self.voice = true;
            }
        } else {
            self.above_ms = 0;
            self.below_ms = self.below_ms.saturating_add(elapsed_ms);
            if self.below_ms >= cfg.silence_ms as u64 {
                self.voice = false;
            }
        }
    }

    pub fn clear_detector(&mut self) {
        self.voice = false;
        self.above_ms = 0;
        self.below_ms = 0;
    }
}

/// Techo absoluto de ganancia de grupo, incluyendo Programa solo si está en su ruta.
pub fn attenuation(base: f32, parent: f32, ceiling: f32) -> f32 {
    let normal = base * parent;
    if normal <= ceiling || normal <= 0.0 {
        1.0
    } else {
        ceiling / normal
    }
}

#[cfg(test)]
#[path = "mic_tests.rs"]
mod tests;
