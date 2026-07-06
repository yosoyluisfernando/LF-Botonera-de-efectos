/// Modulo: cmd_grid.rs
/// Proposito: comandos IPC para gestionar botones de la pestana activa.
use super::AppState;
use crate::domain::button::defaults::new_button;
use crate::domain::button::random_folder;
use crate::domain::colors::{color_palette, random_color, text_for_theme, ColorOption};
use crate::domain::grid::view::paleta_to_grid;
use crate::engine::audio::formats::{probe_duration_secs, validate_audio_file, AUDIO_EXTENSIONS};
use crate::engine::persist::config_io as config;
use crate::model::grid::GridState;
use crate::model::{AppConfig, PaletaData};
use serde::Serialize;

#[derive(Serialize)]
pub struct ButtonStyleSuggestion {
    pub color_bg: String,
    pub color_text: String,
}

/// Devuelve una referencia mutable a la paleta activa del perfil activo.
pub(crate) fn active_paleta(cfg: &mut AppConfig) -> Result<&mut PaletaData, String> {
    cfg.active_paleta_mut()
        .ok_or("Pestaña activa no encontrada".to_string())
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
                b.duration_str = format!("{secs:.1}s");
                changed = true;
            } else {
                b.duration = -1.0;
            }
        }
    }
    let grid = paleta_to_grid(paleta);
    if changed {
        config::save_config(&cfg)?;
    }
    Ok(grid)
}

#[tauri::command]
pub fn assign_file_to_button(
    index: u32,
    path: Option<String>,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let file_path = pick_or_use_file(path)?;
    if std::path::Path::new(&file_path).is_dir() {
        return assign_folder(index, file_path, state);
    }
    validate_audio_file(&file_path)?;
    let stem = file_stem_upper(&file_path);
    let (duration, duration_str) = read_duration(&file_path);
    let mut cfg = state.config.lock().unwrap();
    let theme = cfg.theme.clone();
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    let bg = random_color();
    let text = text_for_theme(&bg, &theme, "button");
    let mut btn = new_button(&paleta.id, index, &stem, &bg, &text);
    btn.path = file_path;
    btn.duration = duration;
    btn.duration_str = duration_str;
    paleta.botones.push(btn);
    save_grid(&mut cfg)
}

#[tauri::command]
pub fn suggest_button_style(state: tauri::State<AppState>) -> ButtonStyleSuggestion {
    let cfg = state.config.lock().unwrap();
    let bg = random_color();
    ButtonStyleSuggestion {
        color_text: text_for_theme(&bg, &cfg.theme, "button"),
        color_bg: bg,
    }
}

#[tauri::command]
pub fn get_color_palette() -> Vec<ColorOption> {
    color_palette()
}

fn assign_folder(
    index: u32,
    folder: String,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    random_folder::ensure_has_audio(&folder)?;
    let stem = file_stem_upper(&folder);
    let mut cfg = state.config.lock().unwrap();
    let theme = cfg.theme.clone();
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    let bg = random_color();
    let text = text_for_theme(&bg, &theme, "button");
    let mut btn = new_button(&paleta.id, index, &stem, &bg, &text);
    btn.type_field = "random_folder".to_string();
    btn.folder = folder;
    btn.duration_str = "RND".to_string();
    paleta.botones.push(btn);
    save_grid(&mut cfg)
}

#[tauri::command]
pub fn clear_button(index: u32, state: tauri::State<AppState>) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let has_button = {
        let paleta = active_paleta(&mut cfg)?;
        paleta.botones.iter().any(|b| b.index == index)
    };
    if has_button {
        state.history.lock().unwrap().remember(&cfg);
    }
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    save_grid(&mut cfg)
}

fn pick_or_use_file(path: Option<String>) -> Result<String, String> {
    path.or_else(|| {
        rfd::FileDialog::new()
            .add_filter("Audio", AUDIO_EXTENSIONS)
            .pick_file()
            .map(|p| p.to_string_lossy().to_string())
    })
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
    if secs > 0.0 {
        (secs, format!("{secs:.1}s"))
    } else {
        (-1.0, String::new())
    }
}

pub(crate) fn save_grid(cfg: &mut AppConfig) -> Result<GridState, String> {
    let grid = paleta_to_grid(active_paleta(cfg)?);
    config::save_config(cfg)?;
    Ok(grid)
}
