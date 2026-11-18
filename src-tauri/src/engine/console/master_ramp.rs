//! Transición compartida del máster: reloj común para todas las tarjetas, sin locks de audio.
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;
const DURATION_US: u64 = 500_000;
pub struct MasterRamp {
    clock: Instant,
    end_us: AtomicU64,
    from: AtomicU32,
}
impl Default for MasterRamp {
    fn default() -> Self {
        Self { clock: Instant::now(), end_us: AtomicU64::new(0), from: AtomicU32::new(0) }
    }
}
impl MasterRamp {
    pub fn reduce(&self, from: f32) {
        self.from.store(from.to_bits(), Ordering::Relaxed);
        self.end_us.store(self.now_us() + DURATION_US, Ordering::Release);
    }
    pub fn cancel(&self) { self.end_us.store(0, Ordering::Release); }
    pub fn gain(&self, target: f32) -> f32 {
        let end = self.end_us.load(Ordering::Acquire);
        if end == 0 { return target; }
        let now = self.now_us();
        if now >= end {
            let _ = self.end_us.compare_exchange(end, 0, Ordering::AcqRel, Ordering::Relaxed);
        }
        self.at(target, end, now)
    }
    fn now_us(&self) -> u64 { self.clock.elapsed().as_micros() as u64 }
    fn at(&self, target: f32, end: u64, now: u64) -> f32 {
        if now >= end { return target; }
        let remaining = (end - now).min(DURATION_US) as f32 / DURATION_US as f32;
        let from = f32::from_bits(self.from.load(Ordering::Relaxed));
        1.0 + (from - 1.0) * remaining
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_clock_reaches_unity_in_half_a_second_without_advancing_per_output() {
        let ramp = MasterRamp::default();
        ramp.reduce(1.5);
        let end = ramp.end_us.load(Ordering::Acquire);
        assert_eq!(ramp.at(1.0, end, end - DURATION_US), 1.5);
        for _ in 0..8 { assert_eq!(ramp.at(1.0, end, end - 250_000), 1.25); }
        assert_eq!(ramp.at(1.0, end, end), 1.0);
        assert_eq!(ramp.at(1.0, end, end + 10), 1.0);
        ramp.cancel();
        assert_eq!(ramp.gain(0.25), 0.25);
    }
}
