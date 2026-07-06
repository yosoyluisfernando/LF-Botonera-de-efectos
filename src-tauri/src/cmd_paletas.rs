/// Módulo: cmd_paletas.rs
/// Propósito: Comandos IPC de PESTAÑAS (paletas). Separado de cmd_profiles.rs
/// por responsabilidad única (perfiles vs pestañas). Reutiliza el helper
/// `next_id` de cmd_profiles para no duplicarlo.
use super::AppState;
use crate::cmd_profiles::next_id;
use crate::domain::grid::reorder as grid_reorder;
use crate::domain::grid::resize as grid_resize;
use crate::engine::input::keyboard as global_shortcuts;
use crate::engine::input::rules as shortcut_rules;
use crate::engine::persist::config_io as config;
use crate::model::{AppConfig, PaletaData};

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
        return Err("Pestaña no encontrada".to_string());
    }
    profile.active_paleta_id = paleta_id;
    config::save_config(&cfg)?;
    let next = cfg.clone();
    drop(cfg);
    crate::engine::cache::warm::warm_visible_tab(&state);
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
#[allow(clippy::too_many_arguments)]
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
        .ok_or("Pestaña no encontrada")?;
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
        .ok_or("Pestaña no encontrada")?;
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
