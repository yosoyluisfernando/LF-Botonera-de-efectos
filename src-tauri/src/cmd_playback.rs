/// Modulo: cmd_playback.rs
/// Proposito: comandos IPC para modo base y toggle SOLO de reproduccion.
use super::AppState;
use crate::engine::persist::config_io as config;
use crate::playback_mode::PlaybackMode;
use serde::Serialize;

#[derive(Serialize)]
pub struct PlaybackState {
    pub mode: String,
    pub solo: bool,
}

#[tauri::command]
pub fn get_playback_mode(state: tauri::State<AppState>) -> String {
    get_playback_state(state)
        .map(|state| state.mode)
        .unwrap_or_else(|_| PlaybackMode::Normal.as_str().to_string())
}

#[tauri::command]
pub fn get_playback_state(state: tauri::State<AppState>) -> Result<PlaybackState, String> {
    let cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let profile = cfg
        .profiles
        .iter()
        .find(|p| p.id == pid)
        .ok_or("active_profile_not_found".to_string())?;
    let legacy_solo = profile.audio.playback_mode == "stop_others";
    Ok(PlaybackState {
        mode: PlaybackMode::from_config(&profile.audio.playback_mode)
            .as_str()
            .to_string(),
        solo: profile.audio.solo_mode || legacy_solo,
    })
}

#[tauri::command]
pub fn set_playback_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mode = PlaybackMode::parse(&mode)?;
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    if let Some(profile) = cfg.profiles.iter_mut().find(|p| p.id == pid) {
        profile.audio.playback_mode = mode.as_str().to_string();
    }
    config::save_config(&cfg)
}

#[tauri::command]
pub fn set_solo_mode(enabled: bool, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    if let Some(profile) = cfg.profiles.iter_mut().find(|p| p.id == pid) {
        profile.audio.solo_mode = enabled;
        if profile.audio.playback_mode == "stop_others" {
            profile.audio.playback_mode = PlaybackMode::Normal.as_str().to_string();
        }
    }
    config::save_config(&cfg)
}
