/// Módulo: cmd_profiles.rs
/// Propósito: Comandos IPC para gestionar perfiles y pestañas.

use super::AppState;
use crate::config;
use crate::types::{AppConfig, AudioConfig, PaletaData, ProfileData};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn next_id(prefix: &str, existing: &[String]) -> String {
    let mut i = 1u32;
    loop {
        let candidate = format!("{}_{}", prefix, i);
        if !existing.contains(&candidate) { return candidate; }
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
    cfg.is_first_boot          = false;
    cfg.weather_module_enabled = weather_enabled;
    cfg.lf_automatizador_link  = link_enabled;
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

// ─── Perfiles ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_active_profile(id: String, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    if !cfg.profiles.iter().any(|p| p.id == id) {
        return Err("Perfil no encontrado".to_string());
    }
    cfg.active_profile_id = id;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn create_profile(name: String, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let ids: Vec<String> = cfg.profiles.iter().map(|p| p.id.clone()).collect();
    let id = next_id("perfil", &ids);
    let paleta_id = format!("{}_paleta_1", id);
    let paleta = PaletaData {
        id: paleta_id.clone(), nombre: "Principal".to_string(),
        rows: 5, cols: 5,
        audio_out: String::new(), shortcut: String::new(),
        tab_bg: String::new(), tab_text: String::new(),
        botones: Vec::new(),
    };
    cfg.profiles.push(ProfileData {
        id, name, bg: String::new(), text: String::new(),
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
    if cfg.profiles.len() <= 1 { return Err("only_profile".to_string()); }
    cfg.profiles.retain(|p| p.id != id);
    if cfg.active_profile_id == id {
        cfg.active_profile_id = cfg.profiles[0].id.clone();
    }
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn update_profile_meta(
    id: String, name: String, bg: String, text: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let p = cfg.profiles.iter_mut().find(|p| p.id == id)
        .ok_or("Perfil no encontrado")?;
    p.name = name; p.bg = bg; p.text = text;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

// ─── Pestañas ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_active_paleta(
    profile_id: String, paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg.profiles.iter_mut().find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    if !profile.paletas.iter().any(|p| p.id == paleta_id) {
        return Err("Pestaña no encontrada".to_string());
    }
    profile.active_paleta_id = paleta_id;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn create_paleta(
    profile_id: String, nombre: String,
    rows: Option<u32>, cols: Option<u32>,
    tab_bg: Option<String>, tab_text: Option<String>, audio_out: Option<String>,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg.profiles.iter_mut().find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    let ids: Vec<String> = profile.paletas.iter().map(|p| p.id.clone()).collect();
    let id = next_id(&format!("{}_paleta", profile_id), &ids);
    profile.paletas.push(PaletaData {
        id: id.clone(), nombre,
        rows: rows.unwrap_or(5), cols: cols.unwrap_or(5),
        audio_out: audio_out.unwrap_or_default(), shortcut: String::new(),
        tab_bg: tab_bg.unwrap_or_default(), tab_text: tab_text.unwrap_or_default(),
        botones: Vec::new(),
    });
    profile.active_paleta_id = id;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn delete_paleta(
    profile_id: String, paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg.profiles.iter_mut().find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    // Código de error: la UI lo traduce con t('errors.only_tab') (Regla 6)
    if profile.paletas.len() <= 1 { return Err("only_tab".to_string()); }
    profile.paletas.retain(|p| p.id != paleta_id);
    if profile.active_paleta_id == paleta_id {
        profile.active_paleta_id = profile.paletas[0].id.clone();
    }
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn update_paleta_meta(
    profile_id: String, paleta_id: String,
    nombre: String, rows: u32, cols: u32,
    tab_bg: Option<String>, tab_text: Option<String>,
    audio_out: Option<String>, shortcut: Option<String>,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg.profiles.iter_mut().find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    let paleta = profile.paletas.iter_mut().find(|p| p.id == paleta_id)
        .ok_or("Pestaña no encontrada")?;
    paleta.nombre = nombre; paleta.rows = rows; paleta.cols = cols;
    if let Some(v) = tab_bg    { paleta.tab_bg    = v; }
    if let Some(v) = tab_text  { paleta.tab_text  = v; }
    if let Some(v) = audio_out { paleta.audio_out = v; }
    if let Some(v) = shortcut  { paleta.shortcut  = v; }
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}
