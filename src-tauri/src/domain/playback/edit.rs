//! Modulo: domain/playback/edit.rs
//! Proposito: resolver el recorte (cue), la ganancia y la duracion efectiva de
//! una pista a partir de lo que el editor guardo en `tracks.db`. Fuente unica:
//! la usan tanto los botones como el reproductor auxiliar. Depende solo del
//! almacen de pistas, no del AppState, para poder llamarse desde el motor.
use crate::engine::dsp::analysis::file_stamp;
use crate::engine::persist::tracks::TrackStore;
use std::sync::Mutex;

/// Cue + ganancia + duracion efectiva resueltos desde tracks.db. Si no hay fila
/// o el archivo cambio (mtime/size), devuelve valores neutros (sin edicion).
pub struct ResolvedEdit {
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    pub file_gain: f32,
    pub duration: f64,
}

pub fn resolve_edit(tracks: &Mutex<TrackStore>, path: &str, fallback_dur: f64) -> ResolvedEdit {
    let converted_duration = crate::engine::audio::transcode::resolve(path).ok()
        .filter(|resolved| resolved != path)
        .map(|_| crate::engine::audio::formats::probe_duration_secs(path))
        .filter(|duration| *duration > 0.0);
    let fallback_dur = converted_duration.unwrap_or(fallback_dur);
    let neutral = ResolvedEdit {
        cue_start_s: 0.0,
        cue_end_s: None,
        file_gain: 1.0,
        duration: fallback_dur,
    };
    let meta = match tracks.lock().unwrap().get(path) {
        Ok(Some(m)) => m,
        _ => return neutral,
    };
    let (mtime, size) = file_stamp(path);
    if !meta.matches(mtime, size) {
        return neutral;
    }
    let (cue_start_s, cue_end_s) = meta.sanitized_cue();
    let eff = converted_duration.map(|duration| {
        let end = cue_end_s.unwrap_or(duration).min(duration);
        (end - cue_start_s).max(0.0)
    }).unwrap_or_else(|| meta.effective_duration_s());
    ResolvedEdit {
        cue_start_s,
        cue_end_s,
        file_gain: meta.effective_gain_linear(),
        duration: if eff > 0.0 { eff } else { fallback_dur },
    }
}
