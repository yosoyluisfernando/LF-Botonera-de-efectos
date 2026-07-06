/// Módulo: preload_cache.rs
/// Propósito: caché en RAM de PCM decodificado (precarga). HashMap por archivo +
/// LRU + presupuesto de bytes. Al disparar, `build_play_source` sirve desde RAM
/// si el archivo está cacheado (sin I/O) o decodifica perezoso como hasta hoy.
/// La caché se LLENA en etapas posteriores (preloader, ver PLAN_PRECARGA.md);
/// aquí queda la infraestructura y el enganche al motor.
use crate::engine::audio::decode::{self as audio_decode, BoxSource};
use crate::engine::cache::cached_source::{CachedPcm, CachedSource};
use crate::engine::dsp::cue_source;
use crate::engine::persist::db;
use rodio::Source;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

const MB: usize = 1024 * 1024;

/// Caché LRU acotada por RAM. Clave = ruta normalizada (igual que el editor).
pub struct PreloadCache {
    map: HashMap<String, Arc<CachedPcm>>,
    order: VecDeque<String>,
    bytes_used: usize,
    budget_bytes: usize,
}

impl PreloadCache {
    pub fn new(budget_mb: u32) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            bytes_used: 0,
            budget_bytes: budget_mb as usize * MB,
        }
    }

    /// Cambia el presupuesto en caliente y expulsa lo que ya no quepa.
    pub fn set_budget(&mut self, budget_mb: u32) {
        self.budget_bytes = budget_mb as usize * MB;
        self.evict();
    }

    /// Vacía todo lo precargado en RAM.
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.bytes_used = 0;
    }

    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// (bytes usados, nº de archivos) — para diagnóstico en Ajustes.
    pub fn stats(&self) -> (usize, usize) {
        (self.bytes_used, self.map.len())
    }

    /// Devuelve el PCM y lo marca como recién usado (LRU).
    pub fn get(&mut self, key: &str) -> Option<Arc<CachedPcm>> {
        let pcm = self.map.get(key)?.clone();
        self.bump(key);
        Some(pcm)
    }

    /// Inserta PCM; si no cabe en el presupuesto total, no se cachea.
    pub fn insert(&mut self, key: String, pcm: CachedPcm) {
        self.insert_arc(key, Arc::new(pcm));
    }

    /// Inserta PCM ya compartido por otra cache. Evita clonar muestras grandes.
    pub fn insert_arc(&mut self, key: String, pcm: Arc<CachedPcm>) {
        let bytes = pcm.bytes();
        if bytes == 0 || bytes > self.budget_bytes {
            return;
        }
        self.remove(&key);
        self.bytes_used += bytes;
        self.map.insert(key.clone(), pcm);
        self.order.push_back(key);
        self.evict();
    }

    fn bump(&mut self, key: &str) {
        if let Some(i) = self.order.iter().position(|k| k == key) {
            if let Some(k) = self.order.remove(i) {
                self.order.push_back(k);
            }
        }
    }

    fn remove(&mut self, key: &str) {
        if let Some(pcm) = self.map.remove(key) {
            self.bytes_used = self.bytes_used.saturating_sub(pcm.bytes());
            if let Some(i) = self.order.iter().position(|k| k == key) {
                self.order.remove(i);
            }
        }
    }

    fn evict(&mut self) {
        while self.bytes_used > self.budget_bytes {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(pcm) = self.map.remove(&oldest) {
                self.bytes_used = self.bytes_used.saturating_sub(pcm.bytes());
            }
        }
    }
}

/// Decodifica un archivo a PCM i16 (para precargar). Va en hilo de comando o del
/// preloader, NUNCA en el hilo de audio.
pub fn decode_pcm(path: &str) -> Option<CachedPcm> {
    let src = audio_decode::source_from_path(path, false)?;
    let channels = src.channels().max(1);
    let sample_rate = src.sample_rate().max(1);
    let data: Vec<i16> = src.map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16).collect();
    if data.is_empty() {
        return None;
    }
    Some(CachedPcm {
        data,
        channels,
        sample_rate,
    })
}

/// Fuente a reproducir: desde la caché (RAM, sin I/O) si el archivo está; si no,
/// decodificación perezosa como hasta ahora. Cue y bucle se aplican igual.
pub fn build_play_source(
    cache: &Arc<Mutex<PreloadCache>>,
    path: &str,
    loop_mode: bool,
    cue_start_s: f64,
    cue_end_s: Option<f64>,
) -> Option<BoxSource> {
    let hit = cache.lock().unwrap().get(&db::normalize_key(path));
    match hit {
        // Cache: arranca en el offset (seek O(1)); el cue_end pasa a ser
        // relativo a ese inicio. Sin descartar muestras una a una.
        Some(pcm) => Some(cue_source::cued_from(
            CachedSource::new_at(pcm, cue_start_s),
            loop_mode,
            0.0,
            cue_end_s.map(|e| (e - cue_start_s).max(0.0)),
        )),
        None => cue_source::cued_source(path, loop_mode, cue_start_s, cue_end_s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm(samples: usize) -> CachedPcm {
        CachedPcm {
            data: vec![0i16; samples],
            channels: 1,
            sample_rate: 8000,
        }
    }

    #[test]
    fn evicts_least_recently_used_over_budget() {
        // Presupuesto 1 MB → caben 512k muestras i16. Cada bloque = 200k muestras.
        let mut c = PreloadCache::new(1);
        c.insert("a".into(), pcm(200_000));
        c.insert("b".into(), pcm(200_000));
        c.get("a"); // 'a' pasa a ser el más reciente
        c.insert("c".into(), pcm(200_000)); // 600k > 512k → expulsa el LRU ('b')
        assert!(c.contains("a"));
        assert!(c.contains("c"));
        assert!(!c.contains("b"));
    }

    #[test]
    fn item_larger_than_budget_is_not_cached() {
        let mut c = PreloadCache::new(1);
        c.insert("big".into(), pcm(700_000)); // ~1.4 MB > 1 MB
        assert!(!c.contains("big"));
        assert_eq!(c.stats().1, 0);
    }

    #[test]
    fn clear_releases_cached_items() {
        let mut c = PreloadCache::new(1);
        c.insert("a".into(), pcm(10));
        c.clear();
        assert_eq!(c.stats(), (0, 0));
        assert!(!c.contains("a"));
    }
}
