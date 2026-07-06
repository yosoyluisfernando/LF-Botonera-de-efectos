/// Orquestador del analisis del editor con progreso y cache persistente.
use crate::core::AppState;
use crate::domain::track_response::{response_from, response_from_envelope, AnalyzeResponse};
use crate::engine::cache::track_analysis::CachedTrackAnalysis;
use crate::engine::cache::waveform_disk;
use crate::engine::dsp::{analysis as audio_analysis, cue_detect};
use crate::engine::persist::db;
use crate::model::track::TrackMeta;
use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[derive(Clone, Serialize)]
struct Progress<'a> {
    path: &'a str,
    stage: &'a str,
}

pub fn analyze_track(
    app: tauri::AppHandle,
    path: String,
    buckets: usize,
) -> Result<AnalyzeResponse, String> {
    emit(&app, &path, "cache");
    let state = app.state::<AppState>();
    let key = db::normalize_key(&path);
    let (mtime, size) = audio_analysis::file_stamp(&path);
    let (norm, cue, wave_cfg, preload) = {
        let cfg = state.config.lock().unwrap();
        (
            cfg.norm.clone(),
            cfg.cue_detect.clone(),
            cfg.waveform_cache.clone(),
            cfg.preload.clone(),
        )
    };
    if let Some(hit) = state.track_analysis.lock().unwrap().get(&key, mtime, size) {
        let merged = state
            .tracks
            .lock()
            .unwrap()
            .get(&path)?
            .unwrap_or_else(|| hit.meta.clone());
        state
            .waveforms
            .lock()
            .unwrap()
            .put(&key, Arc::clone(&hit.envelope));
        let detected = cue_detect::detect_boundaries(
            &hit.pcm.data,
            hit.pcm.sample_rate,
            hit.pcm.channels,
            &cue,
        );
        emit(&app, &path, "ready");
        return Ok(response_from(&hit, merged, buckets, detected));
    }
    if let Some(row) = valid_track_row(&state, &path, mtime, size)? {
        if let Some(env) = waveform_disk::load(&path, mtime, size) {
            let env = Arc::new(env);
            state.waveforms.lock().unwrap().put(&key, Arc::clone(&env));
            let _ = waveform_disk::cleanup(&wave_cfg);
            emit(&app, &path, "ready");
            return Ok(response_from_envelope(
                &env,
                &row,
                row.clone(),
                buckets,
                (None, None),
            ));
        }
        emit(&app, &path, "waveform");
        let wf = audio_analysis::analyze_waveform_only(&path, &cue)?;
        waveform_disk::save(&path, mtime, size, &wf.envelope)?;
        let env = Arc::new(wf.envelope);
        state.waveforms.lock().unwrap().put(&key, Arc::clone(&env));
        let _ = waveform_disk::cleanup(&wave_cfg);
        emit(&app, &path, "ready");
        return Ok(response_from_envelope(
            &env,
            &row,
            row.clone(),
            buckets,
            (wf.auto_cue_start_s, wf.auto_cue_end_s),
        ));
    }
    emit(&app, &path, "decode");
    let analysis = audio_analysis::analyze(&path, &norm, &cue)?;
    emit(&app, &path, "save");
    state.tracks.lock().unwrap().upsert(&analysis.meta)?;
    waveform_disk::save(
        &path,
        analysis.meta.mtime,
        analysis.meta.size,
        &analysis.envelope,
    )?;
    let envelope = Arc::new(analysis.envelope);
    let pcm = Arc::new(analysis.pcm);
    if preload.enabled && analysis.meta.duration_s <= preload.max_duration_s as f64 {
        state
            .audio
            .lock()
            .unwrap()
            .preload_cache_handle()
            .lock()
            .unwrap()
            .insert_arc(key.clone(), Arc::clone(&pcm));
    }
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
    emit(&app, &path, "cleanup");
    let _ = waveform_disk::cleanup(&wave_cfg);
    emit(&app, &path, "ready");
    Ok(response_from(&item, merged, buckets, detected))
}

fn valid_track_row(
    state: &tauri::State<AppState>,
    path: &str,
    mtime: i64,
    size: i64,
) -> Result<Option<TrackMeta>, String> {
    let row = state.tracks.lock().unwrap().get(path)?;
    Ok(row.filter(|r| {
        r.mtime == mtime
            && r.size == size
            && r.duration_s > 0.0
            && r.measured_peak_db.is_some()
            && r.measured_lufs.is_some()
    }))
}

fn emit(app: &tauri::AppHandle, path: &str, stage: &str) {
    let _ = app.emit("track-analysis-progress", Progress { path, stage });
}
