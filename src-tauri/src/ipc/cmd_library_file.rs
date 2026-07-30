use super::AppState;
use crate::domain::library::metadata_fields::LibraryMetadataItem;
use crate::engine::persist::db;
use serde::Serialize;

#[derive(Serialize)]
pub struct LibraryRenameResponse {
    pub old_path: String,
    pub new_path: String,
    pub item: LibraryMetadataItem,
}

#[tauri::command]
pub async fn library_rename_file(
    path: String,
    new_file_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryRenameResponse, String> {
    super::cmd_library_metadata::ensure_not_playing(&state, &path, "library_rename_locked")?;
    let service = state.library.clone();
    let config = state.config.clone();
    let requested = new_file_name.clone();
    let old = path.clone();
    let (old_path, new_path) = tauri::async_runtime::spawn_blocking(move || {
        let mut config = config.lock().map_err(|_| "config_lock")?;
        service.rename_file(&mut config, &old, &requested)
    })
    .await
    .map_err(|error| error.to_string())??;
    invalidate_caches(&state, &old_path, &new_path);
    super::cmd_player_queue::sync_queue(&state);
    let item = state
        .library
        .metadata_get(std::slice::from_ref(&new_path))?
        .items
        .into_iter()
        .next()
        .ok_or("library_track_not_found")?;
    Ok(LibraryRenameResponse {
        old_path,
        new_path,
        item,
    })
}

fn invalidate_caches(state: &AppState, old_path: &str, new_path: &str) {
    let old_key = db::normalize_key(old_path);
    let new_key = db::normalize_key(new_path);
    if let Ok(audio) = state.audio.lock() {
        if let Ok(mut cache) = audio.preload_cache_handle().lock() {
            cache.remove(&old_key);
            cache.remove(&new_key);
        }
    }
    if let Ok(mut cache) = state.track_analysis.lock() {
        cache.remove(&old_key);
        cache.remove(&new_key);
    }
    if let Ok(mut cache) = state.waveforms.lock() {
        cache.remove(&old_key);
        cache.remove(&new_key);
    }
}
