/// MÃ³dulo: cmd_profiles.rs
/// PropÃ³sito: Comandos IPC para gestionar perfiles y pestaÃ±as.
use super::AppState;
use crate::cmd_master_volume;
use crate::config;
use crate::global_shortcuts;
use crate::grid_reorder;
use crate::grid_resize;
use crate::shortcut_rules;
use crate::types::{AppConfig, AudioConfig, PaletaData, ProfileData};

// â”€â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn next_id(prefix: &str, existing: &[String]) -> String {
    let mut i = 1u32;
    loop {
        let candidate = format!("{}_{}", prefix, i);
        if !existing.contains(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

// â”€â”€â”€ Config general â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€â”€ Perfiles â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
    // CÃ³digo de error: la UI lo traduce con t('errors.only_profile') (Regla 6)
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

// â”€â”€â”€ PestaÃ±as â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub fn set_active_paleta(
    profile_id: String,
    paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    if !profile.paletas.iter().any(|p| p.id == paleta_id) {
        return Err("PestaÃ±a no encontrada".to_string());
    }
    profile.active_paleta_id = paleta_id;
    config::save_config(&cfg)?;
    let next = cfg.clone();
    drop(cfg);
    crate::preload_warm::warm_visible_tab(&state);
    Ok(next)
}

#[tauri::command]
pub fn create_paleta(
    profile_id: String,
    nombre: String,
    rows: Option<u32>,
    cols: Option<u32>,
    tab_bg: Option<String>,
    tab_text: Option<String>,
    audio_out: Option<String>,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    let ids: Vec<String> = profile.paletas.iter().map(|p| p.id.clone()).collect();
    let id = next_id(&format!("{}_paleta", profile_id), &ids);
    profile.paletas.push(PaletaData {
        id: id.clone(),
        nombre,
        rows: rows.unwrap_or(5),
        cols: cols.unwrap_or(5),
        audio_out: audio_out.unwrap_or_default(),
        shortcut: String::new(),
        tab_bg: tab_bg.unwrap_or_default(),
        tab_text: tab_text.unwrap_or_default(),
        botones: Vec::new(),
    });
    profile.active_paleta_id = id;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn delete_paleta(
    profile_id: String,
    paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let idx = cfg
        .profiles
        .iter()
        .position(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    if cfg.profiles[idx].paletas.len() <= 1 {
        return Err("only_tab".to_string());
    }
    if !cfg.profiles[idx].paletas.iter().any(|p| p.id == paleta_id) {
        return Err("Pestaña no encontrada".to_string());
    }
    state.history.lock().unwrap().remember(&cfg);
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    profile.paletas.retain(|p| p.id != paleta_id);
    if profile.active_paleta_id == paleta_id {
        profile.active_paleta_id = profile.paletas[0].id.clone();
    }
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn update_paleta_meta(
    profile_id: String,
    paleta_id: String,
    nombre: String,
    rows: u32,
    cols: u32,
    tab_bg: Option<String>,
    tab_text: Option<String>,
    audio_out: Option<String>,
    shortcut: Option<String>,
    replace_shortcut: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let mut id_mappings = Vec::new();
    let mut grid_changed = false;
    if let Some(v) = shortcut.as_ref() {
        shortcut_rules::apply_tab_shortcut(
            &mut cfg,
            &profile_id,
            &paleta_id,
            v,
            replace_shortcut.unwrap_or(false),
        )?;
    }
    let paleta_snapshot = cfg
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .and_then(|p| p.paletas.iter().find(|p| p.id == paleta_id))
        .ok_or("PestaÃ±a no encontrada")?;
    let should_resize = paleta_snapshot.rows != rows || paleta_snapshot.cols != cols;
    if should_resize {
        grid_resize::validate_resize(paleta_snapshot, rows, cols)?;
        state.history.lock().unwrap().remember(&cfg);
    }
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    let paleta = profile
        .paletas
        .iter_mut()
        .find(|p| p.id == paleta_id)
        .ok_or("PestaÃ±a no encontrada")?;
    if should_resize {
        let resized = grid_resize::resize_paleta(paleta, rows, cols)?;
        id_mappings = resized.mappings;
        grid_changed = resized.changed;
    }
    paleta.nombre = nombre;
    if let Some(v) = tab_bg {
        paleta.tab_bg = v;
    }
    if let Some(v) = tab_text {
        paleta.tab_text = v;
    }
    if let Some(v) = audio_out {
        paleta.audio_out = v;
    }
    if let Some(v) = shortcut {
        paleta.shortcut = v;
    }
    config::save_config(&cfg)?;
    drop(cfg);
    if grid_changed {
        grid_reorder::remap_active_audio_ids(&state, &id_mappings);
    }
    global_shortcuts::sync(&app)?;
    let cfg = state.config.lock().unwrap();
    Ok(cfg.clone())
}
