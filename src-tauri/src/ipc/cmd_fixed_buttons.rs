use super::cmd_fixed_panel::{
    button_prefix, buttons_mut, ensure_capacity, next_index, state, FixedPanelState,
};
use super::AppState;
use crate::domain::button::audio_file;
use crate::domain::button::defaults::new_button_preserving_colors;
use crate::engine::audio::formats::AUDIO_EXTENSIONS;
use crate::engine::persist::config_io;
#[tauri::command(async)]
pub fn assign_file_to_fixed_button(
    index: Option<u32>,
    path: Option<String>,
    state_: tauri::State<AppState>,
) -> Result<FixedPanelState, String> {
    let path = match path {
        Some(path) => path,
        None => crate::engine::files::FileDialog::new()
            .add_filter("Audio", AUDIO_EXTENSIONS)
            .pick_file()?
            .map(|p| p.to_string_lossy().into_owned())
            .ok_or("Operación cancelada.")?,
    };
    let is_folder = std::path::Path::new(&path).is_dir();
    if is_folder {
        crate::domain::button::random_folder::ensure_has_audio(&path)?;
    }
    let (theme, previous) = {
        let mut cfg = state_.config.lock().unwrap();
        let old = if let Some(i) = index {
            buttons_mut(&mut cfg)?
                .iter()
                .find(|b| b.index == i)
                .cloned()
        } else {
            None
        };
        (cfg.theme.clone(), old)
    };
    let prepared = if is_folder {
        None
    } else {
        Some(audio_file::from_audio_file_preserving(
            "pending",
            1,
            path.clone(),
            &theme,
            previous.as_ref(),
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
        let mut button =
            new_button_preserving_colors(&prefix, index, &name, &theme, previous.as_ref());
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
