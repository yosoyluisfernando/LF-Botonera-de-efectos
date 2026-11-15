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
    start: usize,
    end: usize,
    pos: usize,
    loop_mode: bool,
}

impl CachedSource {
    /// Delimita la región en frames completos para mantener el orden de canales.
    pub fn new(pcm: Arc<CachedPcm>, start_s: f64, end_s: Option<f64>, loop_mode: bool) -> Self {
        let channels = usize::from(pcm.channels.max(1));
        let frames = pcm.data.len() / channels;
        let sample_at = |seconds: f64| {
            ((seconds * f64::from(pcm.sample_rate.max(1))) as usize).min(frames) * channels
        };
        let start = sample_at(start_s);
        let end = end_s.map_or(frames * channels, sample_at).max(start);
        Self {
            pcm,
            start,
            end,
            pos: start,
            loop_mode,
        }
    }
}

impl Iterator for CachedSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.pos == self.end {
            if !self.loop_mode || self.start == self.end {
                return None;
            }
            self.pos = self.start;
        }
        let s = self.pcm.data[self.pos];
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
        let out: Vec<f32> = CachedSource::new(pcm, 0.0, None, false).collect();
        assert_eq!(out.len(), 4);
        assert!((out[0] - 0.0).abs() < 1e-6);
        assert!((out[3] - 1.0).abs() < 1e-3);
        assert!(out[2] < 0.0);
    }

    #[test]
    fn loop_repeats_only_the_cued_stereo_frames() {
        let pcm = Arc::new(CachedPcm {
            data: vec![0, 0, 32767, -32767, -32767, 32767, 0, 0],
            channels: 2,
            sample_rate: 10,
        });
        let out: Vec<f32> = CachedSource::new(pcm, 0.15, Some(0.35), true)
            .take(10)
            .collect();
        assert_eq!(out, [1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0]);
    }

    #[test]
    fn empty_loop_ends_without_reading_past_the_pcm() {
        let pcm = Arc::new(CachedPcm {
            data: vec![1, 2],
            channels: 2,
            sample_rate: 10,
        });
        assert_eq!(CachedSource::new(pcm, 1.0, None, true).next(), None);
    }

    #[test]
    fn bytes_counts_two_per_sample() {
        let pcm = CachedPcm {
            data: vec![1, 2, 3],
            channels: 1,
            sample_rate: 8000,
        };
        assert_eq!(pcm.bytes(), 6);
    }
}
