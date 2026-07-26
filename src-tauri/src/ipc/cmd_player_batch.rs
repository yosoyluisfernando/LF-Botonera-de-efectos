//! Adición atómica de una o varias rutas a la cola del reproductor.
use super::cmd_player::{player_view, PlayerView};
use super::cmd_player_queue::{insert_track, sync_queue};
use super::AppState;
use crate::domain::button::audio_file;
use crate::engine::persist::config_io;
use std::collections::HashSet;

pub(crate) fn add_paths(
    state: &AppState,
    paths: Vec<String>,
    index: Option<u32>,
) -> Result<PlayerView, String> {
    let theme = state.config.lock().unwrap().theme.clone();
    let additions = prepare_paths(paths, &theme)?;
    add_prepared(state, additions, index)
}

fn prepare_paths(paths: Vec<String>, theme: &str) -> Result<Vec<crate::model::ButtonData>, String> {
    if paths.is_empty() || paths.iter().any(|path| path.is_empty()) {
        return Err("player_empty_path".into());
    }
    paths
        .into_iter()
        .map(|path| audio_file::from_audio_file("player", 1, path, theme))
        .collect()
}

fn add_prepared(
    state: &AppState,
    mut additions: Vec<crate::model::ButtonData>,
    index: Option<u32>,
) -> Result<PlayerView, String> {
    let mut cfg = state.config.lock().unwrap();
    let mut ids = cfg
        .player
        .tracks
        .iter()
        .map(|track| track.id.clone())
        .collect::<HashSet<_>>();
    let mut candidate = 1usize;
    for button in &mut additions {
        let id = loop {
            let id = format!("player_btn_{candidate}");
            candidate += 1;
            if ids.insert(id.clone()) {
                break id;
            }
        };
        button.id = id;
    }
    let mut position = index;
    for button in additions {
        insert_track(&mut cfg.player.tracks, button, position);
        position = position.map(|value| value + 1);
    }
    config_io::save_config(&cfg)?;
    drop(cfg);
    sync_queue(state);
    Ok(player_view(state))
}

#[tauri::command]
pub async fn player_add_tracks(
    paths: Vec<String>,
    index: Option<u32>,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerView, String> {
    let theme = state.config.lock().unwrap().theme.clone();
    let additions = tauri::async_runtime::spawn_blocking(move || prepare_paths(paths, &theme))
        .await
        .map_err(|error| error.to_string())??;
    add_prepared(&state, additions, index)
}
