//! Atenuación temporal independiente del fader. Persiste al reconstruir el grafo.
use crate::domain::mic::attenuation;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

#[derive(Default)]
pub struct DuckControl {
    active: AtomicBool,
    ceiling: AtomicU32,
    down_ms: AtomicU32,
    up_ms: AtomicU32,
    wet: AtomicU32,
}

impl DuckControl {
    pub fn set(&self, active: bool, ceiling: f32, down: u32, up: u32) {
        self.ceiling.store(ceiling.to_bits(), Ordering::Relaxed);
        self.down_ms.store(down, Ordering::Relaxed);
        self.up_ms.store(up, Ordering::Relaxed);
        self.active.store(active, Ordering::Relaxed);
    }

    fn advance(&self, rate: u32) -> f32 {
        let active = self.active.load(Ordering::Relaxed);
        let wet = f32::from_bits(self.wet.load(Ordering::Relaxed));
        if (!active && wet == 0.0) || (active && wet == 1.0) {
            return wet;
        }
        let ms = if active { &self.down_ms } else { &self.up_ms }.load(Ordering::Relaxed);
        let step = if ms == 0 {
            1.0
        } else {
            1000.0 / (ms as f32 * rate as f32)
        };
        let wet = if active {
            (wet + step).min(1.0)
        } else {
            (wet - step).max(0.0)
        };
        self.wet.store(wet.to_bits(), Ordering::Relaxed);
        wet
    }
}

pub struct DuckStage {
    pub control: Arc<DuckControl>,
    pub parent: Option<Arc<AtomicU32>>,
    pub parent_ramp: Option<Arc<super::master_ramp::MasterRamp>>,
}

impl DuckStage {
    pub fn gain(&self, base: f32, rate: u32) -> f32 {
        let wet = self.control.advance(rate);
        if wet == 0.0 {
            return base;
        }
        let parent = self
            .parent
            .as_ref()
            .map(|p| f32::from_bits(p.load(Ordering::Relaxed)))
            .unwrap_or(1.0);
        let parent = self.parent_ramp.as_ref().map(|r| r.gain(parent)).unwrap_or(parent);
        let cap = f32::from_bits(self.control.ceiling.load(Ordering::Relaxed));
        base * (1.0 - wet * (1.0 - attenuation(base, parent, cap)))
    }
}

#[cfg(test)]
#[path = "duck_tests.rs"]
mod tests;
