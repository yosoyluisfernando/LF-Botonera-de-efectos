/// Módulo: cmd_tracks.rs
/// Propósito: comandos IPC del editor de pistas. La UI es un control remoto: no
/// hace DSP ni SQL, solo pide análisis/ventanas de onda y guarda cue/dB.
use super::AppState;
use crate::audio_analysis;
use crate::db;
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

/// Analiza un archivo (decodifica una vez): pico, LUFS, onda. Persiste lo medido
/// preservando las ediciones del usuario y cachea el envolvente para el zoom.
#[tauri::command]
pub fn analyze_track(
    path: String,
    buckets: usize,
    state: tauri::State<AppState>,
) -> Result<AnalyzeResponse, String> {
    let analysis = audio_analysis::analyze(&path)?;
    {
        let store = state.tracks.lock().unwrap();
        store.upsert(&analysis.meta)?;
    }
    // Cachea el PCM ya decodificado → la previa/scrubbing del editor hace seek
    // O(1) (sin descartar muestras una a una). Reúsa la decodificación.
    let cache = state.audio.lock().unwrap().preload_cache_handle();
    cache.lock().unwrap().insert(db::normalize_key(&path), analysis.pcm);

    let merged = state
        .tracks
        .lock()
        .unwrap()
        .get(&path)?
        .unwrap_or_else(|| analysis.meta.clone());

    let duration_s = analysis.envelope.duration_s();
    let waveform = analysis.envelope.view(0.0, duration_s, buckets);
    let resp = AnalyzeResponse {
        peak_db: analysis.meta.measured_peak_db,
        lufs: analysis.meta.measured_lufs,
        suggested_norm_db: analysis.meta.norm_gain_db,
        duration_s,
        waveform,
        meta: merged,
    };

    let key = db::normalize_key(&path);
    state
        .waveforms
        .lock()
        .unwrap()
        .put(&key, Arc::new(analysis.envelope));
    Ok(resp)
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
