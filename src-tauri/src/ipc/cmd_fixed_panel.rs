use super::AppState;
use crate::domain::grid::view::button_to_view;
use crate::engine::persist::config_io;
use crate::model::{AppConfig, ButtonData, ButtonView, FixedPanelConfig};
use serde::Serialize;

#[derive(Serialize)]
pub struct FixedPanelState { pub settings: FixedPanelConfig, pub buttons: Vec<ButtonView> }

pub(super) fn buttons(cfg: &AppConfig) -> &[ButtonData] {
    if cfg.fixed_panel.scope == "profile" {
        cfg.active_profile().map(|p| p.fixed_buttons.as_slice()).unwrap_or(&[])
    } else { &cfg.fixed_panel.global_buttons }
}

/// Prefijo de id de los botones fijos segun el alcance activo.
pub(super) fn button_prefix(cfg: &AppConfig) -> String {
    if cfg.fixed_panel.scope == "profile" {
        format!("fixed_{}", cfg.active_profile_id)
    } else { "fixed_global".into() }
}

pub(super) fn buttons_mut(cfg: &mut AppConfig) -> Result<&mut Vec<ButtonData>, String> {
    if cfg.fixed_panel.scope == "profile" {
        Ok(&mut cfg.active_profile_mut().ok_or("active_profile_not_found")?.fixed_buttons)
    } else { Ok(&mut cfg.fixed_panel.global_buttons) }
}

pub(super) fn state(cfg: &AppConfig) -> FixedPanelState {
    let mut settings = cfg.fixed_panel.clone();
    // Migracion: la antigua vista "list" pasa a ser el modo reproductor.
    if settings.view == "list" {
        settings.view = "player".into();
    }
    FixedPanelState { settings, buttons: buttons(cfg).iter().map(button_to_view).collect() }
}

#[tauri::command]
pub fn get_fixed_panel(state_: tauri::State<AppState>) -> FixedPanelState {
    state(&state_.config.lock().unwrap())
}

#[tauri::command]
pub fn set_fixed_panel_settings(
    scope: String, view: String, side: String, visible: bool,
    show_on_start: bool, columns: u32, row_mode: String, rows: u32,
    width: u32, modes_position: String, state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    if !matches!(scope.as_str(), "global" | "profile")
        || !matches!(view.as_str(), "player" | "buttons")
        || !matches!(side.as_str(), "left" | "right")
        || !matches!(row_mode.as_str(), "unlimited" | "limited")
        || !matches!(modes_position.as_str(), "top" | "bottom")
        || !(1..=5).contains(&columns) || !(1..=20).contains(&rows)
        || !(180..=600).contains(&width) {
        return Err("invalid_fixed_panel_settings".into());
    }
    let mut cfg = state_.config.lock().unwrap();
    if row_mode == "limited" && buttons(&cfg).len() > (columns * rows) as usize {
        return Err("fixed_panel_capacity_too_small".into());
    }
    cfg.fixed_panel.scope = scope; cfg.fixed_panel.view = view; cfg.fixed_panel.side = side;
    cfg.fixed_panel.visible = visible; cfg.fixed_panel.show_on_start = show_on_start;
    cfg.fixed_panel.columns = columns; cfg.fixed_panel.row_mode = row_mode;
    cfg.fixed_panel.rows = rows; cfg.fixed_panel.width = width;
    cfg.fixed_panel.modes_position = modes_position;
    config_io::save_config(&cfg)?; Ok(state(&cfg))
}

/// Indice para anexar al final del panel fijo del alcance activo (max+1).
/// Fuente unica del calculo; la interfaz nunca lo decide.
pub(super) fn next_index(cfg: &AppConfig) -> u32 {
    buttons(cfg).iter().map(|b| b.index).max().map_or(1, |m| m + 1)
}

pub(super) fn ensure_capacity(cfg: &AppConfig, replacing: bool) -> Result<(), String> {
    if !replacing && cfg.fixed_panel.row_mode == "limited"
        && buttons(cfg).len() >= (cfg.fixed_panel.columns * cfg.fixed_panel.rows) as usize {
        return Err("fixed_panel_full".into());
    }
    Ok(())
}

#[tauri::command]
pub fn clear_fixed_scope(scope: String, state_: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state_.config.lock().unwrap();
    match scope.as_str() {
        "global" => cfg.fixed_panel.global_buttons.clear(),
        "profile" => cfg.profiles.iter_mut().for_each(|p| p.fixed_buttons.clear()),
        _ => return Err("invalid_fixed_panel_scope".into()),
    }
    config_io::save_config(&cfg)
}

#[tauri::command]
pub fn reorder_fixed_buttons(
    from_index: u32, to_index: u32, state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap();
    let prefix = button_prefix(&cfg);
    let list = buttons_mut(&mut cfg)?;
    let source = list.iter().position(|b| b.index == from_index).ok_or("button_not_found")?;
    let target = list.iter().position(|b| b.index == to_index);
    let mut mappings = vec![(list[source].id.clone(), format!("{prefix}_btn_{to_index}"))];
    list[source].index = to_index; list[source].id = mappings[0].1.clone();
    if let Some(target) = target {
        mappings.push((list[target].id.clone(), format!("{prefix}_btn_{from_index}")));
        list[target].index = from_index; list[target].id = mappings[1].1.clone();
    }
    config_io::save_config(&cfg)?; let next = state(&cfg); drop(cfg);
    let audio = state_.audio.lock().unwrap();
    crate::domain::playback::state::remap_button_ids(
        audio.button_states_handle(), audio.last_pressed_handle(), &mappings);
    Ok(next)
}
