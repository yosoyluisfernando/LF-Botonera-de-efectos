use super::cmd_fixed_panel::{button_prefix, buttons_mut, ensure_capacity, next_index, state, FixedPanelState};
use super::AppState;
use crate::domain::button::defaults::new_button;
use crate::engine::audio::formats::{probe_duration_secs, validate_audio_file, AUDIO_EXTENSIONS};
use crate::engine::persist::config_io;

#[tauri::command]
pub fn assign_file_to_fixed_button(
    index: Option<u32>, path: Option<String>, state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let path = path.or_else(|| rfd::FileDialog::new().add_filter("Audio", AUDIO_EXTENSIONS)
        .pick_file().map(|p| p.to_string_lossy().to_string())).ok_or("Operación cancelada.")?;
    let is_folder = std::path::Path::new(&path).is_dir();
    if is_folder { crate::domain::button::random_folder::ensure_has_audio(&path)?; }
    else { validate_audio_file(&path)?; }
    let name = std::path::Path::new(&path).file_stem().unwrap_or_default()
        .to_string_lossy().to_uppercase();
    let mut cfg = state_.config.lock().unwrap();
    let index = index.unwrap_or_else(|| next_index(&cfg));
    let replacing = buttons_mut(&mut cfg)?.iter().any(|b| b.index == index);
    ensure_capacity(&cfg, replacing)?;
    let bg = crate::domain::colors::random_color();
    let text = crate::domain::colors::text_for_theme(&bg, &cfg.theme, "button");
    let prefix = button_prefix(&cfg);
    let list = buttons_mut(&mut cfg)?; list.retain(|b| b.index != index);
    let mut btn = new_button(&prefix, index, &name, &bg, &text);
    if is_folder { btn.type_field = "random_folder".into(); btn.folder = path; btn.duration_str = "RND".into(); }
    else { btn.path = path; btn.duration = probe_duration_secs(&btn.path);
        btn.duration_str = if btn.duration > 0.0 { format!("{:.1}s", btn.duration) } else { String::new() }; }
    list.push(btn); config_io::save_config(&cfg)?; Ok(state(&cfg))
}

#[tauri::command]
pub fn clear_fixed_button(index: u32, state_: tauri::State<AppState>) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap(); buttons_mut(&mut cfg)?.retain(|b| b.index != index);
    config_io::save_config(&cfg)?; Ok(state(&cfg))
}

/// Cambio atomico de bandera de reproduccion de un boton fijo. Hermano de
/// `toggle_button_flag`, pero opera sobre la lista fija segun el alcance activo.
#[tauri::command]
pub fn toggle_fixed_button_flag(
    index: u32, flag: String, state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap();
    let btn = buttons_mut(&mut cfg)?.iter_mut().find(|b| b.index == index).ok_or("button_not_found")?;
    match flag.as_str() {
        "loop_mode" => btn.loop_mode = !btn.loop_mode,
        "overlap" => btn.overlap = !btn.overlap,
        "stop_other" => btn.stop_other = !btn.stop_other,
        "restart" => btn.restart = !btn.restart,
        _ => return Err("invalid_button_flag".into()),
    }
    config_io::save_config(&cfg)?; Ok(state(&cfg))
}

#[tauri::command]
pub fn update_fixed_button(
    index: u32, label: String, color_bg: String, color_text: String,
    btn_type: Option<String>, path: Option<String>, folder: Option<String>, vol: Option<f32>,
    shortcut: Option<String>, app: tauri::AppHandle, state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap();
    if let Some(kind) = btn_type.as_deref() { crate::domain::button::types::validate_enabled(&cfg, kind)?; }
    if let Some(value) = vol { if !value.is_finite() || !(0.0..=16.0).contains(&value) { return Err("invalid_volume".into()); } }
    if let Some(value) = path.as_deref() { if !value.is_empty() { validate_audio_file(value)?; } }
    if btn_type.as_deref() == Some("random_folder") {
        if let Some(value) = folder.as_deref() { if !value.is_empty() { crate::domain::button::random_folder::ensure_has_audio(value)?; } }
    }
    let replacing = buttons_mut(&mut cfg)?.iter().any(|b| b.index == index);
    ensure_capacity(&cfg, replacing)?;
    let prefix = button_prefix(&cfg); let list = buttons_mut(&mut cfg)?;
    if !list.iter().any(|b| b.index == index) { list.push(new_button(&prefix, index, &label, &color_bg, &color_text)); }
    let btn = list.iter_mut().find(|b| b.index == index).ok_or("button_not_found")?;
    btn.label = label.clone(); btn.name = label; btn.color_bg = color_bg; btn.color_text = color_text;
    if let Some(value) = btn_type { btn.type_field = value; }
    if let Some(value) = path { if value != btn.path { btn.duration = 0.0; btn.duration_str.clear(); } btn.path = value; }
    if let Some(value) = folder { btn.folder = value; } if let Some(value) = vol { btn.vol = value; }
    if let Some(value) = shortcut { btn.shortcut = value; }
    config_io::save_config(&cfg)?; let next = state(&cfg); drop(cfg);
    crate::engine::input::keyboard::sync(&app)?; Ok(next)
}
