/// Módulo: cmd_grid.rs
/// Propósito: Comandos IPC para gestionar botones de la pestaña activa.

use super::AppState;
use crate::cmd_audio::probe_duration_secs;
use crate::colors::random_color;
use crate::config;
use crate::types::{AppConfig, ButtonData, GridState, PaletaData};

/// Devuelve una referencia mutable a la paleta activa del perfil activo.
pub(crate) fn active_paleta(cfg: &mut AppConfig) -> Result<&mut PaletaData, String> {
    let pid = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    let aid = profile.active_paleta_id.clone();
    profile.paletas.iter_mut()
        .find(|p| p.id == aid)
        .ok_or("Pestaña activa no encontrada".to_string())
}

pub(crate) fn paleta_to_grid(p: &PaletaData) -> GridState {
    GridState { columns: p.cols, rows: p.rows, buttons: p.botones.clone() }
}

#[tauri::command]
pub fn get_grid_state(state: tauri::State<AppState>) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    let mut changed = false;
    for b in paleta.botones.iter_mut() {
        if b.duration == 0.0 && !b.path.is_empty() {
            let secs = probe_duration_secs(&b.path);
            if secs > 0.0 {
                b.duration = secs;
                b.duration_str = format!("{:.1}s", secs);
                changed = true;
            } else {
                b.duration = -1.0;
            }
        }
    }
    let grid = paleta_to_grid(paleta);
    if changed { config::save_config(&cfg)?; }
    Ok(grid)
}

#[tauri::command]
pub fn assign_file_to_button(
    index: u32,
    path: Option<String>,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let file_path = pick_or_use_file(path)?;
    let stem = file_stem_upper(&file_path);
    let (duration, duration_str) = read_duration(&file_path);
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    let mut btn = new_button(&paleta.id, index, &stem, &random_color(), "#FFFFFF");
    btn.path = file_path;
    btn.duration = duration;
    btn.duration_str = duration_str;
    paleta.botones.push(btn);
    save_grid(&mut cfg)
}

#[tauri::command]
pub fn clear_button(index: u32, state: tauri::State<AppState>) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    save_grid(&mut cfg)
}

#[tauri::command]
pub fn reorder_buttons(
    from_index: u32,
    to_index: u32,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    let pid = paleta.id.clone();
    let pos_src = paleta.botones.iter().position(|b| b.index == from_index);
    let pos_dst = paleta.botones.iter().position(|b| b.index == to_index);
    match (pos_src, pos_dst) {
        (Some(a), Some(b)) => {
            move_button(&mut paleta.botones[a], &pid, to_index);
            move_button(&mut paleta.botones[b], &pid, from_index);
        }
        (Some(a), None) => move_button(&mut paleta.botones[a], &pid, to_index),
        _ => {}
    }
    save_grid(&mut cfg)
}

#[tauri::command]
pub fn update_button_data(
    index: u32,
    label: String,
    color_bg: String,
    color_text: String,
    btn_type: Option<String>,
    path: Option<String>,
    folder: Option<String>,
    vol: Option<f32>,
    loop_mode: Option<bool>,
    stop_other: Option<bool>,
    overlap: Option<bool>,
    restart: Option<bool>,
    shortcut: Option<String>,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    ensure_button(paleta, index, &label, &color_bg, &color_text);
    let btn = paleta.botones.iter_mut().find(|b| b.index == index).unwrap();
    btn.label = label.clone();
    btn.name = label;
    btn.color_bg = color_bg;
    btn.color_text = color_text;
    if let Some(v) = btn_type { btn.type_field = v; }
    if let Some(v) = path { set_path(btn, v); }
    if let Some(v) = folder { btn.folder = v; }
    if let Some(v) = vol { btn.vol = v; }
    if let Some(v) = loop_mode { btn.loop_mode = v; }
    if let Some(v) = stop_other { btn.stop_other = v; }
    if let Some(v) = overlap { btn.overlap = v; }
    if let Some(v) = restart { btn.restart = v; }
    if let Some(v) = shortcut { btn.shortcut = v; }
    save_grid(&mut cfg)
}

fn pick_or_use_file(path: Option<String>) -> Result<String, String> {
    path.or_else(|| rfd::FileDialog::new()
        .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a", "wma"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string()))
        .ok_or("Operación cancelada.".to_string())
}

fn file_stem_upper(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase()
}

fn read_duration(path: &str) -> (f64, String) {
    let secs = probe_duration_secs(path);
    if secs > 0.0 { (secs, format!("{:.1}s", secs)) } else { (-1.0, String::new()) }
}

fn save_grid(cfg: &mut AppConfig) -> Result<GridState, String> {
    let grid = paleta_to_grid(active_paleta(cfg)?);
    config::save_config(cfg)?;
    Ok(grid)
}

fn move_button(btn: &mut ButtonData, paleta_id: &str, index: u32) {
    btn.index = index;
    btn.id = format!("{}_btn_{}", paleta_id, index);
}

fn ensure_button(paleta: &mut PaletaData, index: u32, label: &str, bg: &str, text: &str) {
    if paleta.botones.iter().any(|b| b.index == index) { return; }
    paleta.botones.push(new_button(&paleta.id, index, label, bg, text));
}

fn new_button(paleta_id: &str, index: u32, label: &str, bg: &str, text: &str) -> ButtonData {
    ButtonData {
        id: format!("{}_btn_{}", paleta_id, index), index,
        label: label.to_string(), type_field: "audio".to_string(),
        path: String::new(), folder: String::new(), name: label.to_string(),
        color_bg: bg.to_string(), color_text: text.to_string(),
        vol: 1.0, duration: -1.0, duration_str: String::new(),
        loop_mode: false, stop_other: false, overlap: false,
        restart: false, shortcut: String::new(),
    }
}

fn set_path(btn: &mut ButtonData, path: String) {
    if path != btn.path {
        btn.duration = 0.0;
        btn.duration_str = String::new();
    }
    btn.path = path;
}
