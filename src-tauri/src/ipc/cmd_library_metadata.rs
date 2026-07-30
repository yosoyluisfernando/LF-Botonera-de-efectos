use super::AppState;
use crate::domain::library::metadata_fields::{
    LibraryMetadataFields, LibraryMetadataItem, LibraryMetadataResponse,
};

#[tauri::command]
pub async fn library_metadata_get(
    paths: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryMetadataResponse, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.metadata_get(&paths))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_metadata_save(
    paths: Vec<String>,
    fields: LibraryMetadataFields,
    add_tags: Vec<String>,
    remove_tags: Vec<String>,
    write_to_file: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LibraryMetadataItem>, String> {
    let write_to_file = write_to_file.unwrap_or(false);
    if write_to_file {
        let path = paths.first().ok_or("invalid_library_metadata_selection")?;
        ensure_not_playing(&state, path, "library_metadata_write_locked")?;
    }
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.metadata_save(&paths, &fields, &add_tags, &remove_tags, write_to_file)
    })
    .await
    .map_err(|error| error.to_string())?
}

pub(crate) fn ensure_not_playing(
    state: &AppState,
    path: &str,
    error_key: &str,
) -> Result<(), String> {
    use crate::engine::persist::db;
    if state
        .audio
        .lock()
        .map_err(|_| "audio_lock")?
        .is_path_active(path)
    {
        return Err(error_key.into());
    }
    let snapshot = state.player.lock().map_err(|_| "player_lock")?.snapshot();
    let active = snapshot
        .path
        .as_deref()
        .is_some_and(|value| db::normalize_key(value) == db::normalize_key(path));
    (!active).then_some(()).ok_or_else(|| error_key.into())
}

#[tauri::command]
pub async fn library_metadata_suggest_tags(
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.metadata_suggest_tags(&query, limit.unwrap_or(20))
    })
    .await
    .map_err(|error| error.to_string())?
}
