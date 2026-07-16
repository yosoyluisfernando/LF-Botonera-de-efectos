/// Modulo: cmd_profiles.rs
/// Proposito: comandos IPC de perfiles. Las paletas viven en cmd_paletas.rs y
/// la configuracion general en cmd_config.rs.
use super::AppState;
use crate::engine::console::BusId;
use crate::engine::persist::config_io as config;
use crate::ipc::cmd_master_volume;
use crate::model::{AppConfig, AudioConfig, PaletaData, ProfileData};

/// Genera el siguiente id libre con un prefijo (lo usan perfiles y pestanas).
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
    state.console.set_fader(BusId::Programa, volume);
    crate::engine::cache::warm::warm_for_strategy(&state);
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
        fixed_buttons: Vec::new(),
    });
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn delete_profile(id: String, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
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
