use super::cmd_fixed_panel::{
    button_prefix, buttons_mut, ensure_capacity, next_index, state, FixedPanelState,
};
use super::AppState;
use crate::domain::button::audio_file;
use crate::domain::button::defaults::new_button;
use crate::engine::audio::formats::{validate_audio_file, AUDIO_EXTENSIONS};
use crate::engine::persist::config_io;

#[tauri::command]
pub fn assign_file_to_fixed_button(
    index: Option<u32>,
    path: Option<String>,
    state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let path = path
        .or_else(|| {
            rfd::FileDialog::new()
                .add_filter("Audio", AUDIO_EXTENSIONS)
                .pick_file()
                .map(|p| p.to_string_lossy().to_string())
        })
        .ok_or("Operación cancelada.")?;
    let is_folder = std::path::Path::new(&path).is_dir();
    if is_folder {
        crate::domain::button::random_folder::ensure_has_audio(&path)?;
    }
    let prepared = if is_folder {
        None
    } else {
        let theme = state_.config.lock().unwrap().theme.clone();
        Some(audio_file::from_audio_file(
            "pending",
            1,
            path.clone(),
            &theme,
        )?)
    };
    let name = std::path::Path::new(&path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    let mut cfg = state_.config.lock().unwrap();
    let index = index.unwrap_or_else(|| next_index(&cfg));
    let replacing = buttons_mut(&mut cfg)?.iter().any(|b| b.index == index);
    ensure_capacity(&cfg, replacing)?;
    let theme = cfg.theme.clone();
    let prefix = button_prefix(&cfg);
    let list = buttons_mut(&mut cfg)?;
    list.retain(|b| b.index != index);
    let btn = if let Some(mut button) = prepared {
        button.id = format!("{prefix}_btn_{index}");
        button.index = index;
        button
    } else {
        let bg = crate::domain::colors::random_color();
        let text = crate::domain::colors::text_for_theme(&bg, &theme, "button");
        let mut button = new_button(&prefix, index, &name, &bg, &text);
        button.type_field = "random_folder".into();
        button.folder = path;
        button.duration_str = "RND".into();
        button
    };
    list.push(btn);
    config_io::save_config(&cfg)?;
    Ok(state(&cfg))
}

#[tauri::command]
pub fn clear_fixed_button(
    index: u32,
    state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap();
    buttons_mut(&mut cfg)?.retain(|b| b.index != index);
    config_io::save_config(&cfg)?;
    Ok(state(&cfg))
}

/// Cambio atomico de bandera de reproduccion de un boton fijo. Hermano de
/// `toggle_button_flag`, pero opera sobre la lista fija segun el alcance activo.
#[tauri::command]
pub fn toggle_fixed_button_flag(
    index: u32,
    flag: String,
    state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap();
    let btn = buttons_mut(&mut cfg)?
        .iter_mut()
        .find(|b| b.index == index)
        .ok_or("button_not_found")?;
    match flag.as_str() {
        "loop_mode" => btn.loop_mode = !btn.loop_mode,
        "overlap" => btn.overlap = !btn.overlap,
        "stop_other" => btn.stop_other = !btn.stop_other,
        "restart" => btn.restart = !btn.restart,
        _ => return Err("invalid_button_flag".into()),
    }
    config_io::save_config(&cfg)?;
    Ok(state(&cfg))
}

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
    app: tauri::AppHandle,
    state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let mut cfg = state_.config.lock().unwrap();
    if let Some(kind) = btn_type.as_deref() {
        crate::domain::button::types::validate_enabled(&cfg, kind)?;
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
