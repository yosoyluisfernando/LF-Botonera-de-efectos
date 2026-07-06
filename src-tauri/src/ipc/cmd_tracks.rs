use super::AppState;
use crate::engine::dsp::analysis as audio_analysis;
use crate::domain::track_response::{response_from, AnalyzeResponse};
use crate::engine::persist::config_io as config;
use crate::engine::dsp::cue_detect;
use crate::engine::persist::db;
use crate::engine::cache::track_analysis::CachedTrackAnalysis;
use crate::model::track::TrackMeta;
use std::sync::Arc;

#[tauri::command]
pub fn analyze_track(
    path: String,
    buckets: usize,
    state: tauri::State<AppState>,
) -> Result<AnalyzeResponse, String> {
    let (norm, cue_detect) = {
        let cfg = state.config.lock().unwrap();
        (cfg.norm.clone(), cfg.cue_detect.clone())
    };
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
        let detected = cue_detect::detect_boundaries(
            &hit.pcm.data,
            hit.pcm.sample_rate,
            hit.pcm.channels,
            &cue_detect,
        );
        return Ok(response_from(&hit, merged, buckets, detected));
    }

    let analysis = audio_analysis::analyze(&path, &norm, &cue_detect)?;
    {
        let store = state.tracks.lock().unwrap();
        store.upsert(&analysis.meta)?;
    }
    let envelope = Arc::new(analysis.envelope);
    let pcm = Arc::new(analysis.pcm);
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
    let detected = (analysis.auto_cue_start_s, analysis.auto_cue_end_s);
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
    Ok(response_from(&item, merged, buckets, detected))
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
