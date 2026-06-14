/// Módulo: cmd_keys.rs
/// Propósito: Comandos IPC para los atajos globales de teclado (Fase 5):
/// persistencia de las teclas configuradas y navegación cíclica de pestañas.

use super::AppState;
use crate::config;
use crate::types::AppConfig;

/// Guarda los atajos globales (detener todo / pestaña siguiente / anterior)
/// en el perfil activo.
#[tauri::command]
pub fn set_global_keys(
    key_stop: String, key_next: String, key_prev: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    profile.audio.key_stop = key_stop;
    profile.audio.key_next = key_next;
    profile.audio.key_prev = key_prev;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

/// Borra el atajo de un botón específico en cualquier paleta del perfil activo.
#[tauri::command]
pub fn clear_button_shortcut(
    paleta_id: String,
    index: u32,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    if let Some(paleta) = profile.paletas.iter_mut().find(|p| p.id == paleta_id) {
        if let Some(btn) = paleta.botones.iter_mut().find(|b| b.index == index) {
            btn.shortcut = String::new();
            config::save_config(&cfg)?;
        }
    }
    Ok(cfg.clone())
}

/// Cambia la pestaña activa al vecino indicado: offset +1 (siguiente) o
/// -1 (anterior), con comportamiento circular.
#[tauri::command]
pub fn cycle_paleta(offset: i32, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    let len = profile.paletas.len() as i32;
    if len == 0 { return Err("El perfil no tiene pestañas".to_string()); }
    let current = profile.paletas.iter()
        .position(|p| p.id == profile.active_paleta_id)
        .unwrap_or(0) as i32;
    let next = (current + offset).rem_euclid(len) as usize;
    profile.active_paleta_id = profile.paletas[next].id.clone();
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}
