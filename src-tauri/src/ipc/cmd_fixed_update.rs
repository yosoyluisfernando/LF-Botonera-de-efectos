use super::cmd_fixed_panel::{button_prefix, buttons_mut, ensure_capacity, state, FixedPanelState};
use super::AppState;
use crate::domain::button::defaults::new_button;
use crate::engine::audio::formats::validate_audio_file;
use crate::engine::persist::config_io;
#[tauri::command]
pub fn update_fixed_button(
    index: u32,
    label: String,
    color_bg: String,
    color_text: String,
    btn_type: Option<String>,
    path: Option<String>,
    folder: Option<String>,
    vol: Option<f32>,
    shortcut: Option<String>,
    midi: Option<crate::model::MidiBinding>,
    replace_shortcut: Option<bool>,
    visual: Option<crate::model::ButtonVisual>,
    style: Option<crate::model::ButtonStyle>,
    app: tauri::AppHandle,
    state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    if let Some(value) = style.as_ref() {
        value.validate_colors(&color_bg, &color_text)?;
    }
    let mut cfg = state_.config.lock().unwrap();
    if let Some(kind) = btn_type.as_deref() {
        crate::domain::button::types::validate_enabled(&cfg, kind)?;
    }
    if let Some(key) = shortcut.as_deref() {
        let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
        crate::engine::input::rules::block_profile_keys(profile, key)?;
    }
    if let Some(value) = vol {
        if !value.is_finite() || !(0.0..=16.0).contains(&value) {
            return Err("invalid_volume".into());
        }
    }
    if let Some(value) = visual.as_ref() {
        crate::engine::visuals::validate_button_visual(value)?;
    }
    if let Some(value) = midi.as_ref() {
        crate::engine::input::midi_rules::apply_fixed(
            &mut cfg,
            index,
            value,
            replace_shortcut.unwrap_or(false),
        )?;
    }
    if let Some(value) = path.as_deref() {
        if !value.is_empty() {
            validate_audio_file(value)?;
        }
    }
    if btn_type.as_deref() == Some("random_folder") {
        if let Some(value) = folder.as_deref() {
            if !value.is_empty() {
                crate::domain::button::random_folder::ensure_has_audio(value)?;
            }
        }
    }
    let replacing = buttons_mut(&mut cfg)?.iter().any(|b| b.index == index);
    ensure_capacity(&cfg, replacing)?;
    let prefix = button_prefix(&cfg);
    let list = buttons_mut(&mut cfg)?;
    if !list.iter().any(|b| b.index == index) {
        list.push(new_button(&prefix, index, &label, &color_bg, &color_text));
    }
    let btn = list
        .iter_mut()
        .find(|b| b.index == index)
        .ok_or("button_not_found")?;
    btn.label = label.clone();
    btn.name = label;
    btn.color_bg = color_bg;
    btn.color_text = color_text;
    if let Some(value) = style {
        btn.style = value;
    }
    if let Some(value) = btn_type {
        btn.type_field = value;
    }
    if let Some(value) = path {
        if value != btn.path {
            btn.duration = 0.0;
            btn.duration_str.clear();
        }
        btn.path = value;
    }
    if let Some(value) = folder {
        btn.folder = value;
    }
    if let Some(value) = vol {
        btn.vol = value;
    }
    if let Some(value) = shortcut {
        btn.shortcut = value;
    }
    if let Some(value) = midi {
        btn.midi = value;
    }
    if let Some(value) = visual {
        btn.visual = value;
    }
    config_io::save_config(&cfg)?;
    let next = state(&cfg);
    drop(cfg);
    crate::engine::input::keyboard::sync(&app)?;
    Ok(next)
}
