/// Modulo: cmd_button_playback.rs
/// Proposito: reproducir cualquier tipo de boton resolviendo la logica en Rust.
use super::AppState;
use crate::cmd_audio::probe_duration_secs;
use crate::locution_playback;
use crate::playback_mode::{PlaybackFlags, PlaybackMode};
use crate::types::{AppConfig, ButtonData};

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
    state.audio.lock().unwrap().play_file(
        btn.id.clone(),
        &path,
        btn.vol,
        duration,
        flags.loop_mode,
        flags.stop_other,
        flags.overlap,
        flags.restart,
    )
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
