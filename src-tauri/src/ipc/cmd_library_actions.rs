//! Acciones de resultados de Biblioteca compartidas por panel y ventana.
use super::AppState;
use crate::domain::button::audio_file;
use crate::domain::playback::edit::resolve_edit;
use crate::engine::audio::button::PlaybackGroup;
use crate::engine::audio::formats::{probe_duration_secs, validate_audio_file};
use crate::engine::persist::config_io;
use crate::model::{AppConfig, PaletaData};

pub const LIBRARY_LIVE_ID: &str = "__library_live__";

#[tauri::command]
pub fn library_play_live(
    path: String,
    duration_s: Option<f64>,
    position_s: Option<f64>,
    volume: Option<f32>,
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    validate_audio_file(&path)?;
    let fallback = duration_s
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or_else(|| probe_duration_secs(&path));
    let edit = resolve_edit(&state.tracks, &path, fallback);
    let origin = position_s
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
        .clamp(0.0, edit.duration.max(0.0));
    let live_volume = volume.unwrap_or(1.0);
    if !live_volume.is_finite() || !(0.0..=1.5).contains(&live_volume) {
        return Err("invalid_volume".into());
    }
    let fade = state.config.lock().unwrap().fade.clone();
    state.audio.lock().unwrap().enqueue_preload(path.clone());
    state.audio.lock().unwrap().play_file(
        LIBRARY_LIVE_ID.into(),
        &path,
        live_volume,
        edit.duration,
        false,
        false,
        false,
        true,
        edit.cue_start_s + origin,
        edit.cue_end_s,
        edit.file_gain,
        false,
        &fade,
        PlaybackGroup::Main,
    )?;
    state
        .last_played
        .mark(&path, chrono::Utc::now().timestamp());
    Ok(edit.duration)
}

#[tauri::command]
pub async fn library_assign_to_paleta(
    paths: Vec<String>,
    paleta_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppConfig, String> {
    if paths.is_empty() {
        return Err("library_empty_selection".into());
    }
    let theme = state.config.lock().unwrap().theme.clone();
    let mut additions = tauri::async_runtime::spawn_blocking(move || {
        paths
            .into_iter()
            .map(|path| audio_file::from_audio_file("pending", 1, path, &theme))
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .map_err(|error| error.to_string())??;
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg.active_profile_mut().ok_or("active_profile_not_found")?;
    let paleta = profile
        .paletas
        .iter_mut()
        .find(|paleta| paleta.id == paleta_id)
        .ok_or("tab_not_found")?;
    let empty = empty_indices(paleta, additions.len())?;
    for (button, index) in additions.iter_mut().zip(empty) {
        button.index = index;
        button.id = format!("{}_btn_{index}", paleta.id);
    }
    paleta.botones.extend(additions);
    config_io::save_config(&cfg)?;
    Ok(cfg.clone())
}

fn empty_indices(paleta: &PaletaData, needed: usize) -> Result<Vec<u32>, String> {
    let indexes = (1..=paleta.rows * paleta.cols)
        .filter(|index| !paleta.botones.iter().any(|button| button.index == *index))
        .take(needed)
        .collect::<Vec<_>>();
    if indexes.len() != needed {
        return Err("not_enough_empty_buttons".into());
    }
    Ok(indexes)
}

#[cfg(test)]
mod tests {
    use super::empty_indices;
    use crate::domain::button::defaults::new_button;
    use crate::model::PaletaData;

    #[test]
    fn batch_assignment_is_all_or_nothing() {
        let mut paleta = PaletaData {
            id: "p".into(),
            nombre: "Prueba".into(),
            rows: 1,
            cols: 3,
            audio_out: String::new(),
            shortcut: String::new(),
            midi: Default::default(),
            tab_bg: String::new(),
            tab_text: String::new(),
            botones: Vec::new(),
        };
        paleta
            .botones
            .push(new_button("p", 2, "ocupado", "#000", "#fff"));
        assert_eq!(empty_indices(&paleta, 2).unwrap(), vec![1, 3]);
        assert_eq!(
            empty_indices(&paleta, 3).unwrap_err(),
            "not_enough_empty_buttons"
        );
    }
}
