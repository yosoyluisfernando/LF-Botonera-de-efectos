/// Modulo: cmd_playback_progress.rs
/// Proposito: comandos IPC para configurar y controlar la barra de progreso.
use super::AppState;
use crate::config;
use crate::model::playback::PlaybackProgressConfig;

#[tauri::command]
pub fn set_playback_progress_config(
    config_in: PlaybackProgressConfig,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.playback_progress = config_in.sanitized();
    config::save_config(&cfg)
}

#[tauri::command]
pub fn seek_active_playback(
    delta_s: Option<f64>,
    position_s: Option<f64>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state.audio.lock().unwrap().seek_active(delta_s, position_s)
}
