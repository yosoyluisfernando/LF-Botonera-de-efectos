/// Modulo: track_analysis_cache.rs
/// Proposito: cache efimera del analisis completo del editor de pistas.
/// Evita re-decodificar y recalcular LUFS al mover el editor entre modal/ventana.
use crate::engine::cache::cached_source::CachedPcm;
use crate::model::track::TrackMeta;
use crate::engine::dsp::waveform::WaveEnvelope;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub struct CachedTrackAnalysis {
    pub mtime: i64,
    pub size: i64,
    pub meta: TrackMeta,
    pub envelope: Arc<WaveEnvelope>,
    pub pcm: Arc<CachedPcm>,
}

pub struct TrackAnalysisCache {
    map: HashMap<String, Arc<CachedTrackAnalysis>>,
    order: VecDeque<String>,
    cap: usize,
}

impl Default for TrackAnalysisCache {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            cap: 6,
        }
    }
}

impl TrackAnalysisCache {
    pub fn get(&mut self, key: &str, mtime: i64, size: i64) -> Option<Arc<CachedTrackAnalysis>> {
        let hit = self.map.get(key)?.clone();
        if hit.mtime != mtime || hit.size != size {
            self.remove(key);
            return None;
        }
        self.bump(key);
        Some(hit)
    }

    pub fn put(&mut self, key: String, item: CachedTrackAnalysis) -> Arc<CachedTrackAnalysis> {
        self.remove(&key);
        let item = Arc::new(item);
        self.map.insert(key.clone(), item.clone());
        self.order.push_back(key);
        self.evict();
        item
    }

    fn bump(&mut self, key: &str) {
        if let Some(i) = self.order.iter().position(|k| k == key) {
            if let Some(k) = self.order.remove(i) {
                self.order.push_back(k);
            }
        }
    }

    fn remove(&mut self, key: &str) {
        self.map.remove(key);
        if let Some(i) = self.order.iter().position(|k| k == key) {
            self.order.remove(i);
        }
    }

    fn evict(&mut self) {
        while self.order.len() > self.cap {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
    }
}
