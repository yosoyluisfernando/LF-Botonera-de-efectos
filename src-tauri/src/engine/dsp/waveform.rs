/// Módulo: waveform.rs
/// Propósito: envolvente de forma de onda (min/max) preparada para ZOOM sin
/// verse mal. Se calcula al vuelo, vive solo mientras se edita (cache acotada)
/// y nunca se persiste. El lienzo (JS) pide la ventana visible a la resolución
/// del canvas; aquí se agrega el envolvente de alta resolución a esas columnas.
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Mínimo de muestras por punto del envolvente: a 48 kHz ≈ 0,7 ms → zoom nítido.
const MIN_SAMPLES_PER_POINT: usize = 32;

/// Envolvente mono de alta resolución: un par (min,max) por bloque de muestras.
pub struct WaveEnvelope {
    mins: Vec<f32>,
    maxs: Vec<f32>,
    sample_rate: u32,
    frames: usize,
}

impl WaveEnvelope {
    /// Construye el envolvente desde PCM intercalado. `max_points` acota la
    /// memoria en archivos largos; los cortos llegan a MIN_SAMPLES_PER_POINT.
    pub fn build(interleaved: &[f32], channels: u16, sample_rate: u32, max_points: usize) -> Self {
        let ch = channels.max(1) as usize;
        let frames = interleaved.len() / ch;
        let max_points = max_points.max(1);
        let spp = ((frames + max_points - 1) / max_points).max(MIN_SAMPLES_PER_POINT);
        let mut mins = Vec::new();
        let mut maxs = Vec::new();
        let mut i = 0;
        while i < frames {
            let end = (i + spp).min(frames);
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for f in i..end {
                let base = f * ch;
                let mut acc = 0.0f32;
                for c in 0..ch {
                    acc += interleaved[base + c];
                }
                let m = acc / ch as f32; // mezcla mono para el dibujo
                lo = lo.min(m);
                hi = hi.max(m);
            }
            mins.push(if lo.is_finite() { lo } else { 0.0 });
            maxs.push(if hi.is_finite() { hi } else { 0.0 });
            i = end;
        }
        Self {
            mins,
            maxs,
            sample_rate,
            frames,
        }
    }

    /// Duración total en segundos.
    pub fn duration_s(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.frames as f64 / self.sample_rate as f64
    }

    pub(crate) fn from_parts(
        mins: Vec<f32>,
        maxs: Vec<f32>,
        sample_rate: u32,
        frames: usize,
    ) -> Self {
        Self {
            mins,
            maxs,
            sample_rate,
            frames,
        }
    }

    pub(crate) fn parts(&self) -> (&[f32], &[f32], u32, usize) {
        (&self.mins, &self.maxs, self.sample_rate, self.frames)
    }

    /// Agrega la ventana visible [start_s, end_s] en `buckets` columnas. Devuelve
    /// pares intercalados [min,max] por columna (longitud = buckets*2). Al hacer
    /// zoom, la ventana encoge pero las columnas siguen al ancho del canvas, así
    /// el detalle se mantiene nítido hasta la resolución del envolvente.
    pub fn view(&self, start_s: f64, end_s: f64, buckets: usize) -> Vec<f32> {
        let n = self.mins.len();
        let buckets = buckets.max(1);
        let mut out = Vec::with_capacity(buckets * 2);
        if n == 0 {
            out.resize(buckets * 2, 0.0);
            return out;
        }
        let total = self.duration_s().max(1e-9);
        let s = start_s.clamp(0.0, total);
        let e = end_s.clamp(s, total);
        let p0 = (s / total * n as f64).floor() as usize;
        let p1 = ((e / total * n as f64).ceil() as usize).clamp(p0 + 1, n);
        let span = p1 - p0;
        for b in 0..buckets {
            let a = p0 + (b * span) / buckets;
            let mut be = p0 + ((b + 1) * span) / buckets;
            if be <= a {
                be = (a + 1).min(p1);
            }
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for p in a..be {
                lo = lo.min(self.mins[p]);
                hi = hi.max(self.maxs[p]);
            }
            out.push(if lo.is_finite() { lo } else { 0.0 });
            out.push(if hi.is_finite() { hi } else { 0.0 });
        }
        out
    }
}

/// Cache acotada de envolventes por archivo (clave ya normalizada). Mantiene
/// solo los últimos N que se han abierto en el editor; nada se guarda en disco.
pub struct WaveformCache {
    map: HashMap<String, Arc<WaveEnvelope>>,
    order: VecDeque<String>,
    cap: usize,
}

impl Default for WaveformCache {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            cap: 6,
        }
    }
}

impl WaveformCache {
    pub fn get(&self, key: &str) -> Option<Arc<WaveEnvelope>> {
        self.map.get(key).cloned()
    }

    pub fn put(&mut self, key: &str, env: Arc<WaveEnvelope>) {
        if self.map.insert(key.to_string(), env).is_none() {
            self.order.push_back(key.to_string());
            while self.order.len() > self.cap {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_envelope() -> WaveEnvelope {
        // 4000 frames mono, rampa -1..1 → envolvente con varios puntos.
        let samples: Vec<f32> = (0..4000).map(|i| (i as f32 / 2000.0) - 1.0).collect();
        WaveEnvelope::build(&samples, 1, 1000, 120_000)
    }

    #[test]
    fn view_has_requested_bucket_count() {
        let env = ramp_envelope();
        let v = env.view(0.0, env.duration_s(), 256);
        assert_eq!(v.len(), 256 * 2);
    }

    #[test]
    fn zoom_window_stays_within_amplitude_range() {
        let env = ramp_envelope();
        // Una ventana pequeña (zoom) debe seguir devolviendo buckets completos.
        let v = env.view(0.0, 0.5, 512);
        assert_eq!(v.len(), 512 * 2);
        assert!(v.iter().all(|&x| (-1.001..=1.001).contains(&x)));
    }

    #[test]
    fn empty_is_safe() {
        let env = WaveEnvelope::build(&[], 2, 48000, 1000);
        assert_eq!(env.view(0.0, 1.0, 100).len(), 200);
    }
}
