/// Módulo: cue_source.rs
/// Propósito: aplicar el "manual cue" (punto de inicio y recorte de fin) a una
/// fuente de audio contando muestras. Saltar silencios/colas sin reescribir el
/// archivo. Si hay bucle, se repite la región recortada.
use crate::audio_decode::{self, BoxSource};
use rodio::Source;
use std::time::Duration;

/// Fuente que descarta `skip_samples` muestras al inicio y, opcionalmente, corta
/// tras `take_samples`. Cuenta en muestras intercaladas (sensible a canales).
pub struct CuedSource {
    inner: BoxSource,
    channels: u16,
    sample_rate: u32,
    skip_samples: u64,
    take_samples: Option<u64>,
    skipped: u64,
    emitted: u64,
}

impl CuedSource {
    pub fn new(inner: BoxSource, cue_start_s: f64, cue_end_s: Option<f64>) -> Self {
        let channels = inner.channels().max(1);
        let sample_rate = inner.sample_rate().max(1);
        let per_sec = sample_rate as f64 * channels as f64;
        let skip_samples = (cue_start_s.max(0.0) * per_sec) as u64;
        let take_samples = cue_end_s.map(|e| ((e - cue_start_s).max(0.0) * per_sec) as u64);
        Self {
            inner,
            channels,
            sample_rate,
            skip_samples,
            take_samples,
            skipped: 0,
            emitted: 0,
        }
    }
}

impl Iterator for CuedSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        while self.skipped < self.skip_samples {
            self.inner.next()?; // si la fuente acaba durante el salto, fin
            self.skipped += 1;
        }
        if let Some(t) = self.take_samples {
            if self.emitted >= t {
                return None;
            }
        }
        let s = self.inner.next()?;
        self.emitted += 1;
        Some(s)
    }
}

impl Source for CuedSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Aplica cue (inicio/fin) y, si procede, bucle, a una fuente YA construida
/// (p.ej. desde la caché de precarga). Misma lógica que `cued_source` pero sin
/// abrir el archivo, para no duplicar el tratamiento del cue.
pub fn cued_from<S>(base: S, loop_mode: bool, cue_start_s: f64, cue_end_s: Option<f64>) -> BoxSource
where
    S: Source<Item = f32> + Send + 'static,
{
    let has_cue = cue_start_s > 0.0 || cue_end_s.is_some();
    if has_cue {
        let cued = CuedSource::new(Box::new(base), cue_start_s, cue_end_s);
        if loop_mode {
            Box::new(cued.repeat_infinite())
        } else {
            Box::new(cued)
        }
    } else if loop_mode {
        Box::new(base.repeat_infinite())
    } else {
        Box::new(base)
    }
}

/// Construye la fuente final desde un archivo aplicando cue y, si procede, bucle.
/// Sin cue (inicio 0 y sin fin) delega en el camino normal, sin coste extra.
pub fn cued_source(
    path: &str,
    loop_mode: bool,
    cue_start_s: f64,
    cue_end_s: Option<f64>,
) -> Option<BoxSource> {
    let has_cue = cue_start_s > 0.0 || cue_end_s.is_some();
    if !has_cue {
        return audio_decode::source_from_path(path, loop_mode);
    }
    let base = audio_decode::source_from_path(path, false)?;
    let cued = CuedSource::new(base, cue_start_s, cue_end_s);
    if loop_mode {
        Some(Box::new(cued.repeat_infinite()))
    } else {
        Some(Box::new(cued))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    fn ramp(n: usize, ch: u16, sr: u32) -> BoxSource {
        Box::new(SamplesBuffer::new(ch, sr, (0..n).map(|i| i as f32).collect::<Vec<_>>()))
    }

    #[test]
    fn skip_drops_leading_samples() {
        // 20 muestras mono a 10 Hz = 2 s; cue 0,5 s → saltar 5 muestras.
        let cued = CuedSource::new(ramp(20, 1, 10), 0.5, None);
        let out: Vec<f32> = cued.collect();
        assert_eq!(out.len(), 15);
        assert_eq!(out[0], 5.0);
        assert_eq!(out[14], 19.0);
    }

    #[test]
    fn take_trims_tail() {
        // cue 0,5 s (salta 5) y fin 1,0 s → tomar 5 muestras (índices 5..9).
        let cued = CuedSource::new(ramp(20, 1, 10), 0.5, Some(1.0));
        let out: Vec<f32> = cued.collect();
        assert_eq!(out.len(), 5);
        assert_eq!(out[0], 5.0);
        assert_eq!(out[4], 9.0);
    }

    #[test]
    fn stereo_counts_both_channels() {
        // 2 canales a 10 Hz: 0,5 s = 0.5*10*2 = 10 muestras saltadas.
        let cued = CuedSource::new(ramp(40, 2, 10), 0.5, None);
        let out: Vec<f32> = cued.collect();
        assert_eq!(out.len(), 30);
        assert_eq!(out[0], 10.0);
    }
}
