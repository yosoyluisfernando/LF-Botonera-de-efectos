/// Módulo: cached_source.rs
/// Propósito: PCM decodificado en memoria (i16) y la fuente que lo reproduce sin
/// tocar el disco. La mitad de RAM que f32. Sobre esta fuente se aplican cue y
/// ganancia igual que sobre cualquier otra (ver cue_source / master_button).
use rodio::Source;
use std::sync::Arc;
use std::time::Duration;

/// PCM intercalado en i16 listo para disparar sin I/O. Compartido por Arc para
/// que varias reproducciones del mismo archivo no clonen las muestras.
pub struct CachedPcm {
    pub data: Vec<i16>,
    pub channels: u16,
    pub sample_rate: u32,
}

impl CachedPcm {
    /// Tamaño en bytes que ocupa en la caché (2 bytes por muestra i16).
    pub fn bytes(&self) -> usize {
        self.data.len() * std::mem::size_of::<i16>()
    }
}

/// Fuente que reproduce un `CachedPcm` compartido convirtiendo i16→f32 al vuelo.
pub struct CachedSource {
    pcm: Arc<CachedPcm>,
    pos: usize,
}

impl CachedSource {
    pub fn new(pcm: Arc<CachedPcm>) -> Self {
        Self { pcm, pos: 0 }
    }
}

impl Iterator for CachedSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let s = *self.pcm.data.get(self.pos)?;
        self.pos += 1;
        Some(s as f32 / 32767.0)
    }
}

impl Source for CachedSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.pcm.channels.max(1)
    }
    fn sample_rate(&self) -> u32 {
        self.pcm.sample_rate.max(1)
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproduces_all_samples_as_f32() {
        let pcm = Arc::new(CachedPcm {
            data: vec![0, 16384, -16384, 32767],
            channels: 2,
            sample_rate: 48000,
        });
        let out: Vec<f32> = CachedSource::new(pcm).collect();
        assert_eq!(out.len(), 4);
        assert!((out[0] - 0.0).abs() < 1e-6);
        assert!((out[3] - 1.0).abs() < 1e-3);
        assert!(out[2] < 0.0);
    }

    #[test]
    fn bytes_counts_two_per_sample() {
        let pcm = CachedPcm { data: vec![1, 2, 3], channels: 1, sample_rate: 8000 };
        assert_eq!(pcm.bytes(), 6);
    }
}
