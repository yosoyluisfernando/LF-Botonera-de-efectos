/// Modulo: cmd_button_playback.rs
/// Proposito: reproducir cualquier tipo de boton resolviendo la logica en Rust.
use super::AppState;
use crate::ipc::cmd_audio::probe_duration_secs;
use crate::engine::weather::playback as locution_playback;
use crate::domain::playback::mode::{PlaybackFlags, PlaybackMode};
use crate::model::{AppConfig, ButtonData};
use crate::model::preload::PreloadStrategy;

/// Comando IPC para disparar un boton por id desde la interfaz.
#[tauri::command]
pub fn play_button(id: String, state: tauri::State<AppState>) -> Result<(), String> {
    play_button_id(&state, &id)
}

/// Dispara un boton por id; tambien lo usa el manejador de atajos globales.
pub fn play_button_id(state: &AppState, id: &str) -> Result<(), String> {
    let (cfg, btn) = button_snapshot(state, id)?;
    match btn.type_field.as_str() {
        "time" => locution_playback::play_time(
            state,
            &cfg,
            btn.id.clone(),
            btn.vol,
            Some(btn.folder.as_str()),
        ),
        "temperature" | "humidity" => locution_playback::play_climate(
            state,
            &cfg,
            btn.id.clone(),
            btn.type_field.as_str(),
            btn.vol,
            Some(btn.folder.as_str()),
        ),
        "random_folder" => {
            let is_active = is_button_active(state, &btn.id);
            let path = state.random_folders.lock().unwrap().active_or_next_audio(
                &btn.id,
                &btn.folder,
                is_active,
            )?;
            let duration = probe_duration_secs(&path);
            play_file(state, &cfg, &btn, path, duration)
        }
        _ => {
            if btn.path.is_empty() {
                return Err("button_without_audio".to_string());
            }
            play_file(state, &cfg, &btn, btn.path.clone(), btn.duration)
        }
    }
}

fn button_snapshot(state: &AppState, id: &str) -> Result<(AppConfig, ButtonData), String> {
    let cfg = state.config.lock().unwrap().clone();
    let btn = cfg
        .profiles
        .iter()
        .flat_map(|p| &p.paletas)
        .flat_map(|p| &p.botones)
        .find(|b| b.id == id)
        .cloned()
        .ok_or("Boton no encontrado")?;
    Ok((cfg, btn))
}

fn play_file(
    state: &AppState,
    cfg: &AppConfig,
    btn: &ButtonData,
    path: String,
    duration: f64,
) -> Result<(), String> {
    let profile = cfg.profiles.iter().find(|p| p.id == cfg.active_profile_id);
    let mode = profile
        .map(|p| p.audio.playback_mode.as_str())
        .unwrap_or("normal");
    let mut flags = PlaybackMode::from_config(mode).resolve_flags(PlaybackFlags {
        loop_mode: btn.loop_mode,
        stop_other: btn.stop_other,
        overlap: btn.overlap,
        restart: btn.restart,
    });
    if profile.map(|p| p.audio.solo_mode).unwrap_or(false) || mode == "stop_others" {
        flags.stop_other = true;
    }
    // Edición por archivo (cue + ganancia) del editor de pistas, si sigue vigente.
    let edit = resolve_edit(state, &path, duration);
    let button_volume = if btn.type_field == "random_folder" {
        btn.vol
    } else {
        1.0
    };
    let result = state.audio.lock().unwrap().play_file(
        btn.id.clone(),
        &path,
        button_volume,
        edit.duration,
        flags.loop_mode,
        flags.stop_other,
        flags.overlap,
        flags.restart,
        edit.cue_start_s,
        edit.cue_end_s,
        edit.file_gain,
        false, // botones normales → salida principal (al aire)
        &cfg.fade,
    );
    seed_preload(state, cfg, &path, duration);
    result
}

/// Precarga OnPlay + historial: si la precarga está activa, marca la
/// reproducción (debounce a tracks.db) y, en modo "a medida que se reproducen",
/// encola el archivo (si es corto) para que la próxima vez sea instantáneo.
/// `file_dur` es la duración del archivo (no la efectiva del cue).
fn seed_preload(state: &AppState, cfg: &AppConfig, path: &str, file_dur: f64) {
    let p = &cfg.preload;
    if !p.enabled {
        return;
    }
    state.last_played.mark(path, chrono::Utc::now().timestamp());
    if p.strategy == PreloadStrategy::OnPlay && file_dur > 0.0 && file_dur < p.max_duration_s as f64
    {
        state.audio.lock().unwrap().enqueue_preload(path.to_string());
    }
}

/// Cue + ganancia + duración efectiva resueltos desde tracks.db. Si no hay fila
/// o el archivo cambió (mtime/size), devuelve valores neutros (sin edición).
struct ResolvedEdit {
    cue_start_s: f64,
    cue_end_s: Option<f64>,
    file_gain: f32,
    duration: f64,
}

fn resolve_edit(state: &AppState, path: &str, fallback_dur: f64) -> ResolvedEdit {
    let neutral = ResolvedEdit {
        cue_start_s: 0.0,
        cue_end_s: None,
        file_gain: 1.0,
        duration: fallback_dur,
    };
    let meta = match state.tracks.lock().unwrap().get(path) {
        Ok(Some(m)) => m,
        _ => return neutral,
    };
    let (mtime, size) = crate::engine::dsp::analysis::file_stamp(path);
    if !meta.matches(mtime, size) {
        return neutral;
    }
    let (cue_start_s, cue_end_s) = meta.sanitized_cue();
    let eff = meta.effective_duration_s();
    ResolvedEdit {
        cue_start_s,
        cue_end_s,
        file_gain: meta.effective_gain_linear(),
        duration: if eff > 0.0 { eff } else { fallback_dur },
    }
}

fn is_button_active(state: &AppState, id: &str) -> bool {
    let states = state.audio.lock().unwrap().button_states_handle();
    let is_active = states
        .lock()
        .unwrap()
        .get(id)
        .map(|group| group.iter().any(|s| !s.is_done()))
        .unwrap_or(false);
    is_active
}
