/// Módulo: cmd_tracks.rs
/// Propósito: comandos IPC del editor de pistas. La UI es un control remoto: no
/// hace DSP ni SQL, solo pide análisis/ventanas de onda y guarda cue/dB.
use super::AppState;
use crate::audio_analysis;
use crate::config;
use crate::db;
use crate::track_analysis_cache::CachedTrackAnalysis;
use crate::types_track::TrackMeta;
use serde::Serialize;
use std::sync::Arc;

/// Respuesta del análisis inicial al abrir el editor.
#[derive(Serialize)]
pub struct AnalyzeResponse {
    /// Metadatos combinados (medidos ahora + cue/dB previos del usuario).
    pub meta: TrackMeta,
    /// Onda de toda la pista a la resolución pedida (pares [min,max]).
    pub waveform: Vec<f32>,
    pub duration_s: f64,
    pub peak_db: Option<f64>,
    pub lufs: Option<f64>,
    pub suggested_norm_db: f64,
}

fn response_from(item: &CachedTrackAnalysis, meta: TrackMeta, buckets: usize) -> AnalyzeResponse {
    let duration_s = item.envelope.duration_s();
    AnalyzeResponse {
        peak_db: item.meta.measured_peak_db,
        lufs: item.meta.measured_lufs,
        suggested_norm_db: item.meta.norm_gain_db,
        waveform: item.envelope.view(0.0, duration_s, buckets),
        duration_s,
        meta,
    }
}

/// Analiza un archivo (decodifica una vez): pico, LUFS, onda. Persiste lo medido
/// preservando las ediciones del usuario y cachea el envolvente para el zoom.
#[tauri::command]
pub fn analyze_track(
    path: String,
    buckets: usize,
    state: tauri::State<AppState>,
) -> Result<AnalyzeResponse, String> {
    let key = db::normalize_key(&path);
    let (mtime, size) = audio_analysis::file_stamp(&path);
    if let Some(hit) = state.track_analysis.lock().unwrap().get(&key, mtime, size) {
        let merged = {
            let store = state.tracks.lock().unwrap();
            store.get(&path)?.unwrap_or_else(|| hit.meta.clone())
        };
        state
            .waveforms
            .lock()
            .unwrap()
            .put(&key, Arc::clone(&hit.envelope));
        state
            .audio
            .lock()
            .unwrap()
            .preload_cache_handle()
            .lock()
            .unwrap()
            .insert_arc(key, Arc::clone(&hit.pcm));
        return Ok(response_from(&hit, merged, buckets));
    }

    let analysis = audio_analysis::analyze(&path)?;
    {
        let store = state.tracks.lock().unwrap();
        store.upsert(&analysis.meta)?;
    }
    let envelope = Arc::new(analysis.envelope);
    let pcm = Arc::new(analysis.pcm);
    // Cachea el PCM ya decodificado → la previa/scrubbing del editor hace seek
    // O(1) (sin descartar muestras una a una). Reúsa la decodificación.
    let cache = state.audio.lock().unwrap().preload_cache_handle();
    cache
        .lock()
        .unwrap()
        .insert_arc(key.clone(), Arc::clone(&pcm));

    let merged = state
        .tracks
        .lock()
        .unwrap()
        .get(&path)?
        .unwrap_or_else(|| analysis.meta.clone());

    let item = CachedTrackAnalysis {
        mtime: analysis.meta.mtime,
        size: analysis.meta.size,
        meta: analysis.meta,
        envelope,
        pcm,
    };
    let item = state.track_analysis.lock().unwrap().put(key.clone(), item);
    state
        .waveforms
        .lock()
        .unwrap()
        .put(&key, Arc::clone(&item.envelope));
    Ok(response_from(&item, merged, buckets))
}

/// Devuelve la onda de la ventana visible [start_s, end_s] a `buckets` columnas.
/// Usa el envolvente cacheado (zoom/scroll instantáneo); si no está, re-analiza.
#[tauri::command]
pub fn waveform_view(
    path: String,
    start_s: f64,
    end_s: f64,
    buckets: usize,
    state: tauri::State<AppState>,
) -> Result<Vec<f32>, String> {
    let key = db::normalize_key(&path);
    if let Some(env) = state.waveforms.lock().unwrap().get(&key) {
        return Ok(env.view(start_s, end_s, buckets));
    }
    let analysis = audio_analysis::analyze(&path)?;
    let view = analysis.envelope.view(start_s, end_s, buckets);
    state
        .waveforms
        .lock()
        .unwrap()
        .put(&key, Arc::new(analysis.envelope));
    Ok(view)
}

/// Lee los metadatos guardados de un archivo (o None si nunca se editó).
#[tauri::command]
pub fn get_track_meta(
    path: String,
    state: tauri::State<AppState>,
) -> Result<Option<TrackMeta>, String> {
    state.tracks.lock().unwrap().get(&path)
}

/// Guarda el punto de inicio (y fin opcional) del cue.
#[tauri::command]
pub fn set_track_cue(
    path: String,
    start_s: f64,
    end_s: Option<f64>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state.tracks.lock().unwrap().set_cue(&path, start_s, end_s)
}

/// Guarda el trim manual en dB.
#[tauri::command]
pub fn set_track_gain(
    path: String,
    gain_db: f64,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state.tracks.lock().unwrap().set_gain(&path, gain_db)
}

/// Activa o desactiva la normalización automática para este archivo.
#[tauri::command]
pub fn set_track_normalization(
    path: String,
    enabled: bool,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state
        .tracks
        .lock()
        .unwrap()
        .set_normalization(&path, enabled)
}

/// Persiste la preferencia global del editor: modal o ventana flotante.
#[tauri::command]
pub fn set_editor_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    if mode != "modal" && mode != "window" {
        return Err("invalid_editor_mode".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    cfg.editor_mode = mode;
    config::save_config(&cfg)
}
