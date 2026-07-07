/// Módulo: preloader.rs
/// Propósito: hilo de fondo que LLENA la caché de precarga. Recibe rutas por una
/// cola, decodifica a PCM i16 (preload_cache::decode_pcm) e inserta en la caché.
/// NUNCA decodifica en el hilo de audio: el disparo no se bloquea jamás.
use crate::engine::persist::db;
use crate::engine::cache::preload::{self as preload_cache, PreloadCache};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

/// Cola de precarga: la UI/motor encola rutas; un hilo dedicado las decodifica.
pub struct Preloader {
    tx: Sender<String>,
    enabled: Arc<AtomicBool>,
}

impl Preloader {
    /// Arranca el hilo decodificador sobre la caché compartida.
    pub fn start(cache: Arc<Mutex<PreloadCache>>, enabled: Arc<AtomicBool>) -> Self {
        let (tx, rx) = channel::<String>();
        let worker_enabled = Arc::clone(&enabled);
        thread::spawn(move || {
            for path in rx {
                if !worker_enabled.load(Ordering::Relaxed) {
                    continue;
                }
                let key = db::normalize_key(&path);
                if cache.lock().unwrap().contains(&key) {
                    continue; // ya está en caché
                }
                // Decodifica SIN tener el lock de la caché: no bloquea el disparo.
                if let Some(pcm) = preload_cache::decode_pcm(&path) {
                    if worker_enabled.load(Ordering::Relaxed) {
                        cache.lock().unwrap().insert(key, pcm);
                    }
                }
            }
        });
        Self { tx, enabled }
    }

    /// Encola un archivo para precargar (no bloquea; ignora si el hilo murió).
    pub fn enqueue(&self, path: String) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        let _ = self.tx.send(path);
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }
}
