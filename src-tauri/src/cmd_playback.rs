/// Módulo: cmd_playback.rs
/// Propósito: Comandos IPC para leer y escribir el modo de reproducción global.
/// El modo se guarda por perfil en AudioConfig.playback_mode (Regla 2: config por perfil).

use super::AppState;
use crate::config;

/// Devuelve el modo de reproducción del perfil activo.
#[tauri::command]
pub fn get_playback_mode(state: tauri::State<AppState>) -> String {
    let cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    cfg.profiles.iter().find(|p| p.id == pid)
        .map(|p| p.audio.playback_mode.clone())
        .unwrap_or_else(|| "normal".to_string())
}

/// Persiste el modo de reproducción global para el perfil activo.
#[tauri::command]
pub fn set_playback_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    if let Some(p) = cfg.profiles.iter_mut().find(|p| p.id == pid) {
        p.audio.playback_mode = mode;
    }
    config::save_config(&cfg)
}
