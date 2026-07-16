/// Modulo: engine/console/level.rs
/// Proposito: LevelSource — mide el PICO absoluto por ventana de ~1024 muestras
/// (~23ms). Es el medidor de un bus: va DESPUES del mixer, asi que mide la suma
/// real de ese bus y de ningun otro. No hace RMS ni formulas de combinacion:
/// pico puro sobre PCM f32.
use rodio::Source;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

const WINDOW: usize = 1024;

/// Envuelve Source<Item=f32> actualizando el PICO cada ~1024 muestras.
/// Canal L = muestras de indice par; canal R = indice impar (stereo interleaved).
pub struct LevelSource<S: Source<Item = f32>> {
    inner: S,
    level_l: Arc<AtomicU32>,
    level_r: Arc<AtomicU32>,
    channels: u16,
    peak_l: f32,
    peak_r: f32,
    count: usize,
    ch_idx: u16,
}

impl<S: Source<Item = f32>> LevelSource<S> {
    pub fn new(inner: S, level_l: Arc<AtomicU32>, level_r: Arc<AtomicU32>) -> Self {
        let channels = inner.channels().max(1);
        Self {
            inner,
            level_l,
            level_r,
            channels,
            peak_l: 0.0,
            peak_r: 0.0,
            count: 0,
            ch_idx: 0,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for LevelSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let s = self.inner.next()?;
        let abs = s.abs();
        if self.ch_idx == 0 && abs > self.peak_l {
            self.peak_l = abs;
        }
        if (self.ch_idx == 1 || self.channels < 2) && abs > self.peak_r {
            self.peak_r = abs;
        }
        self.ch_idx = (self.ch_idx + 1) % self.channels.max(1);
        self.count += 1;
        if self.count >= WINDOW {
            self.level_l
                .store(self.peak_l.min(1.0).to_bits(), Ordering::Relaxed);
            self.level_r
                .store(self.peak_r.min(1.0).to_bits(), Ordering::Relaxed);
            self.peak_l = 0.0;
            self.peak_r = 0.0;
            self.count = 0;
        }
        Some(s)
    }
}

impl<S: Source<Item = f32>> Source for LevelSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
