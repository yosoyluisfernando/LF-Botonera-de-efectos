/// Modulo: cmd_config.rs
/// Proposito: comandos IPC de configuracion general de la aplicacion.
use super::AppState;
use crate::engine::persist::config_io as config;
use crate::model::AppConfig;

#[tauri::command]
pub fn get_config(state: tauri::State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_first_boot_complete(
    weather_enabled: bool,
    link_enabled: bool,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.is_first_boot = false;
    cfg.weather_module_enabled = weather_enabled;
    cfg.lf_automatizador_link = link_enabled;
    config::save_config(&cfg)
}

#[tauri::command]
pub fn set_theme(theme: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.theme = theme;
    config::save_config(&cfg)
}

#[tauri::command]
pub fn set_language(language: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.language = language;
    config::save_config(&cfg)
}

#[tauri::command]
pub fn set_button_text_size(size: String, state: tauri::State<AppState>) -> Result<(), String> {
    if !matches!(size.as_str(), "small" | "normal" | "large" | "xlarge") {
        return Err("invalid_button_text_size".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    cfg.button_text_size = size;
    config::save_config(&cfg)
}
