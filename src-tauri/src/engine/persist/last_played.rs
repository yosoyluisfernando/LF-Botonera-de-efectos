/// Módulo: last_played.rs
/// Propósito: historial de "última reproducción" SIN sobrecargar el disco. Se
/// marca en memoria al instante y un hilo lo vuelca a tracks.db AGRUPADO
/// (debounce ~30 s, modo WAL). Reutiliza la columna `last_played` (no crea
/// archivos nuevos). Lo usará la expulsión por tiempo de la precarga (§8 del
/// PLAN_PRECARGA.md). touch_last_played solo afecta a filas existentes
/// (archivos abiertos en el editor); el resto se cubre con la LRU en RAM.
use crate::engine::persist::tracks::TrackStore;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const FLUSH_SECS: u64 = 30;

/// Buffer compartido ruta→epoch pendiente de volcar.
pub type Pending = Arc<Mutex<HashMap<String, i64>>>;

/// Acumulador en memoria de reproducciones recientes.
#[derive(Default)]
pub struct LastPlayed {
    pending: Pending,
}

impl LastPlayed {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra una reproducción en memoria (instantáneo, sin tocar disco).
    pub fn mark(&self, path: &str, epoch: i64) {
        self.pending.lock().unwrap().insert(path.to_string(), epoch);
    }

    pub fn handle(&self) -> Pending {
        Arc::clone(&self.pending)
    }
}

/// Vuelca a SQLite el historial pendiente cada FLUSH_SECS (una sola tanda).
pub fn start_flusher(pending: Pending, store: Arc<Mutex<TrackStore>>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(FLUSH_SECS));
        flush(&pending, &store);
    });
}

/// Vuelca inmediatamente lo pendiente (p.ej. al cerrar la ventana).
pub fn flush_now(pending: &Pending, store: &Arc<Mutex<TrackStore>>) {
    flush(pending, store);
}

fn flush(pending: &Pending, store: &Arc<Mutex<TrackStore>>) {
    let drained: Vec<(String, i64)> = {
        let mut p = pending.lock().unwrap();
        if p.is_empty() {
            return;
        }
        p.drain().collect()
    };
    let store = store.lock().unwrap();
    for (path, epoch) in drained {
        let _ = store.touch_last_played(&path, epoch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_keeps_latest_and_drains() {
        let lp = LastPlayed::new();
        lp.mark("a.mp3", 100);
        lp.mark("a.mp3", 200); // misma ruta → gana la última
        lp.mark("b.mp3", 150);
        let drained: HashMap<String, i64> = lp.handle().lock().unwrap().drain().collect();
        assert_eq!(drained.get("a.mp3"), Some(&200));
        assert_eq!(drained.get("b.mp3"), Some(&150));
    }
}
