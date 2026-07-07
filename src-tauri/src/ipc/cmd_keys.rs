/// Módulo: cmd_keys.rs
/// Propósito: Comandos IPC para los atajos globales de teclado (Fase 5):
/// persistencia de las teclas configuradas y navegación cíclica de pestañas.
use super::AppState;
use crate::engine::input::actions as input_actions;
use crate::engine::input::keyboard as global_shortcuts;
use crate::engine::input::rules as shortcut_rules;
use crate::engine::persist::config_io as config;
use crate::model::AppConfig;

/// Guarda los atajos globales (detener todo / pestaña siguiente / anterior)
/// en el perfil activo.
#[tauri::command]
pub fn set_global_keys(
    key_stop: String,
    key_next: String,
    key_prev: String,
    global_keys: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    if [&key_stop, &key_next, &key_prev]
        .iter()
        .any(|key| shortcut_rules::is_reserved_system_key(key))
    {
        return Err("reserved_system_shortcut".to_string());
    }
    let profile = cfg
        .active_profile_mut()
        .ok_or("Perfil activo no encontrado")?;
    profile.audio.key_stop = key_stop;
    profile.audio.key_next = key_next;
    profile.audio.key_prev = key_prev;
    if let Some(v) = global_keys {
        profile.audio.global_keys = v;
    }
    config::save_config(&cfg)?;
    drop(cfg);
    global_shortcuts::sync(&app)?;
    let cfg = state.config.lock().unwrap();
    Ok(cfg.clone())
}

/// Borra el atajo de un botón específico en cualquier paleta del perfil activo.
#[tauri::command]
pub fn clear_button_shortcut(
    paleta_id: String,
    index: u32,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg
        .active_profile_mut()
        .ok_or("Perfil activo no encontrado")?;
    if let Some(paleta) = profile.paletas.iter_mut().find(|p| p.id == paleta_id) {
        if let Some(btn) = paleta.botones.iter_mut().find(|b| b.index == index) {
            btn.shortcut = String::new();
            config::save_config(&cfg)?;
        }
    }
    drop(cfg);
    global_shortcuts::sync(&app)?;
    let cfg = state.config.lock().unwrap();
    Ok(cfg.clone())
}

/// Cambia la pestaña activa al vecino indicado: offset +1 (siguiente) o
/// -1 (anterior), con comportamiento circular.
#[tauri::command]
pub fn cycle_paleta(offset: i32, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    input_actions::cycle_paleta(&mut cfg, offset)?;
    config::save_config(&cfg)?;
    let result = cfg.clone();
    drop(cfg);
    crate::engine::cache::warm::warm_visible_tab(&state);
    Ok(result)
}
