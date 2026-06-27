/// Módulo: cmd_profiles.rs
/// Propósito: Comandos IPC de configuración general y de PERFILES. Las pestañas
/// (paletas) viven en cmd_paletas.rs (responsabilidad separada).
use super::AppState;
use crate::cmd_master_volume;
use crate::config;
use crate::types::{AppConfig, AudioConfig, PaletaData, ProfileData};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Genera el siguiente id libre con un prefijo (lo usan perfiles y pestañas).
pub fn next_id(prefix: &str, existing: &[String]) -> String {
    let mut i = 1u32;
    loop {
        let candidate = format!("{}_{}", prefix, i);
        if !existing.contains(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

// ─── Config general ───────────────────────────────────────────────────────────

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

// ─── Perfiles ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_active_profile(id: String, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    if !cfg.profiles.iter().any(|p| p.id == id) {
        return Err("Perfil no encontrado".to_string());
    }
    cfg.active_profile_id = id;
    let volume = cmd_master_volume::startup_volume(&cfg);
    config::save_config(&cfg)?;
    let next = cfg.clone();
    drop(cfg);
    state.audio.lock().unwrap().set_master_volume(volume);
    crate::preload_warm::warm_for_strategy(&state);
    Ok(next)
}

#[tauri::command]
pub fn create_profile(name: String, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let ids: Vec<String> = cfg.profiles.iter().map(|p| p.id.clone()).collect();
    let id = next_id("perfil", &ids);
    let paleta_id = format!("{}_paleta_1", id);
    let paleta = PaletaData {
        id: paleta_id.clone(),
        nombre: "Principal".to_string(),
        rows: 5,
        cols: 5,
        audio_out: String::new(),
        shortcut: String::new(),
        tab_bg: String::new(),
        tab_text: String::new(),
        botones: Vec::new(),
    };
    cfg.profiles.push(ProfileData {
        id,
        name,
        bg: String::new(),
        text: String::new(),
        audio: AudioConfig::default(),
        active_paleta_id: paleta_id,
        paletas: vec![paleta],
    });
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn delete_profile(id: String, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    // Código de error: la UI lo traduce con t('errors.only_profile') (Regla 6)
    if cfg.profiles.len() <= 1 {
        return Err("only_profile".to_string());
    }
    if !cfg.profiles.iter().any(|p| p.id == id) {
        return Err("Perfil no encontrado".to_string());
    }
    state.history.lock().unwrap().remember(&cfg);
    cfg.profiles.retain(|p| p.id != id);
    if cfg.active_profile_id == id {
        cfg.active_profile_id = cfg.profiles[0].id.clone();
    }
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn update_profile_meta(
    id: String,
    name: String,
    bg: String,
    text: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let p = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Perfil no encontrado")?;
    p.name = name;
    p.bg = bg;
    p.text = text;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}
