//! Eliminación individual o múltiple de filas del reproductor.
//! Ambas órdenes comparten la misma regla y persisten una sola vez por acción.

use super::cmd_player::{player_view, PlayerView};
use super::cmd_player_queue::{reindex, sync_queue};
use super::AppState;
use crate::domain::player::queue_remove::remove_positions;
use crate::engine::persist::config_io;

#[tauri::command]
pub fn player_remove_track(
    index: u32,
    state: tauri::State<AppState>,
) -> Result<PlayerView, String> {
    remove(&[index], &state)
}

#[tauri::command]
pub fn player_remove_tracks(
    indexes: Vec<u32>,
    state: tauri::State<AppState>,
) -> Result<PlayerView, String> {
    remove(&indexes, &state)
}

fn remove(indexes: &[u32], state: &AppState) -> Result<PlayerView, String> {
    {
        let mut cfg = state.config.lock().unwrap();
        remove_positions(&mut cfg.player.tracks, indexes)?;
        reindex(&mut cfg.player.tracks);
        config_io::save_config(&cfg)?;
    }
    sync_queue(state);
    Ok(player_view(state))
}
