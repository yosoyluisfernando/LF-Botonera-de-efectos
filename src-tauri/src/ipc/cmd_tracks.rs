use super::AppState;
use crate::domain::track_response::AnalyzeResponse;
use crate::engine::dsp::analysis as audio_analysis;
use crate::engine::persist::config_io as config;
use crate::engine::persist::db;
use crate::model::track::TrackMeta;
use std::sync::Arc;

#[tauri::command]
pub async fn analyze_track(
    path: String,
    buckets: usize,
    app: tauri::AppHandle,
) -> Result<AnalyzeResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::engine::dsp::editor_analysis::analyze_track(app, path, buckets)
    })
    .await
    .map_err(|e| e.to_string())?
}

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
    let (norm, cue_detect) = {
        let cfg = state.config.lock().unwrap();
        (cfg.norm.clone(), cfg.cue_detect.clone())
    };
    let analysis = audio_analysis::analyze(&path, &norm, &cue_detect)?;
    let view = analysis.envelope.view(start_s, end_s, buckets);
    state
        .waveforms
        .lock()
        .unwrap()
        .put(&key, Arc::new(analysis.envelope));
    Ok(view)
}

#[tauri::command]
pub fn get_track_meta(
    path: String,
    state: tauri::State<AppState>,
) -> Result<Option<TrackMeta>, String> {
    state.tracks.lock().unwrap().get(&path)
}

#[tauri::command]
pub fn set_track_cue(
    path: String,
    start_s: f64,
    end_s: Option<f64>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state.tracks.lock().unwrap().set_cue(&path, start_s, end_s)
}

#[tauri::command]
pub fn set_track_gain(
    path: String,
    gain_db: f64,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state.tracks.lock().unwrap().set_gain(&path, gain_db)
}

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

#[tauri::command]
pub fn recalculate_norm(path: String, state: tauri::State<AppState>) -> Result<f64, String> {
    let norm = state.config.lock().unwrap().norm.clone();
    let mut meta = state
        .tracks
        .lock()
        .unwrap()
        .get(&path)?
        .ok_or("track_not_analyzed")?;
    let peak = meta.measured_peak_db.unwrap_or(-120.0);
    let g = audio_analysis::suggest_gain(meta.measured_lufs, peak, &norm);
    meta.norm_gain_db = g;
    state.tracks.lock().unwrap().upsert(&meta)?;
    Ok(g)
}

#[tauri::command]
pub fn set_editor_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    if mode != "modal" && mode != "window" {
        return Err("invalid_editor_mode".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    cfg.editor_mode = mode;
    config::save_config(&cfg)
}
