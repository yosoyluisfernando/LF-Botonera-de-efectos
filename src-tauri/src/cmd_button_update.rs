/// Modulo: cmd_button_update.rs
/// Proposito: actualizar datos de un boton existente o nuevo.
use super::AppState;
use crate::audio_formats::validate_audio_file;
use crate::button_defaults::new_button;
use crate::button_types;
use crate::cmd_grid::{active_paleta, save_grid};
use crate::model::{AppConfig, ButtonData, PaletaData};
use crate::model::grid::GridState;
use crate::{global_shortcuts, random_folder, shortcut_rules};

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
    replace_shortcut: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    if let Some(v) = btn_type.as_deref() {
        button_types::validate_enabled(&cfg, v)?;
    }
    if let Some(v) = vol {
        validate_volume(v)?;
    }
    let current = active_button(&cfg, index);
    let current_type = current.map(|b| b.type_field.as_str()).unwrap_or("audio");
    let target_type = btn_type.as_deref().unwrap_or(current_type).to_string();
    validate_button_assets(current, &target_type, path.as_deref(), folder.as_deref())?;
    if let Some(v) = shortcut.as_ref() {
        shortcut_rules::apply_button_shortcut(
            &mut cfg,
            index,
            v,
            replace_shortcut.unwrap_or(false),
        )?;
    }
    let paleta = active_paleta(&mut cfg)?;
    ensure_button(paleta, index, &label, &color_bg, &color_text);
    let btn = paleta
        .botones
        .iter_mut()
        .find(|b| b.index == index)
        .unwrap();
    btn.label = label.clone();
    btn.name = label;
    btn.color_bg = color_bg;
    btn.color_text = color_text;
    if let Some(v) = btn_type {
        btn.type_field = v;
    }
    if let Some(v) = path {
        set_path(btn, v);
    }
    if let Some(v) = folder {
        btn.folder = v;
    }
    if let Some(v) = vol {
        btn.vol = v;
    }
    if let Some(v) = loop_mode {
        btn.loop_mode = v;
    }
    if let Some(v) = stop_other {
        btn.stop_other = v;
    }
    if let Some(v) = overlap {
        btn.overlap = v;
    }
    if let Some(v) = restart {
        btn.restart = v;
    }
    if let Some(v) = shortcut {
        btn.shortcut = v;
    }
    let grid = save_grid(&mut cfg)?;
    drop(cfg);
    global_shortcuts::sync(&app)?;
    Ok(grid)
}

fn ensure_button(paleta: &mut PaletaData, index: u32, label: &str, bg: &str, text: &str) {
    if paleta.botones.iter().any(|b| b.index == index) {
        return;
    }
    paleta
        .botones
        .push(new_button(&paleta.id, index, label, bg, text));
}

fn active_button(cfg: &AppConfig, index: u32) -> Option<&ButtonData> {
    let profile = cfg
        .profiles
        .iter()
        .find(|p| p.id == cfg.active_profile_id)?;
    let paleta = profile
        .paletas
        .iter()
        .find(|p| p.id == profile.active_paleta_id)?;
    paleta.botones.iter().find(|b| b.index == index)
}

fn validate_button_assets(
    current: Option<&ButtonData>,
    target_type: &str,
    path: Option<&str>,
    folder: Option<&str>,
) -> Result<(), String> {
    if let Some(v) = path {
        let current_path = current.map(|b| b.path.as_str()).unwrap_or_default();
        if !v.is_empty() && v != current_path {
            validate_audio_file(v)?;
        }
    }
    if target_type == "random_folder" {
        if let Some(v) = folder {
            let current_folder = current.map(|b| b.folder.as_str()).unwrap_or_default();
            if !v.is_empty() && v != current_folder {
                random_folder::ensure_has_audio(v)?;
            }
        }
    }
    Ok(())
}

fn set_path(btn: &mut crate::model::ButtonData, path: String) {
    if path != btn.path {
        btn.duration = 0.0;
        btn.duration_str = String::new();
    }
    btn.path = path;
}

fn validate_volume(volume: f32) -> Result<(), String> {
    if volume.is_finite() && (0.0..=16.0).contains(&volume) {
        Ok(())
    } else {
        Err("invalid_volume".to_string())
    }
}
