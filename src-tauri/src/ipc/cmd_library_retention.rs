//! IPC de retiro reversible y política de retención de la Biblioteca.
use crate::domain::library::protected_tracks;
use crate::engine::library::root_retention::{RetentionSettings, RetiredRoot};
use crate::ipc::AppState;
use tauri::State;

#[tauri::command]
pub async fn library_remove_root(
    root_id: i64,
    state: State<'_, AppState>,
) -> Result<RetiredRoot, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.remove_root(root_id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_restore_root(root_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.restore_root(root_id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_list_retired_roots(
    state: State<'_, AppState>,
) -> Result<Vec<RetiredRoot>, String> {
    let config = state.config.clone();
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let config_guard = config.lock().map_err(|_| "config_lock")?;
        let protected = protected_tracks::from_config(&config_guard);
        service.purge_expired(&protected)?;
        drop(config_guard);
        service.list_retired_roots()
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_get_retention(
    state: State<'_, AppState>,
) -> Result<RetentionSettings, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.retention_settings())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_set_retention(
    retention_days: u16,
    state: State<'_, AppState>,
) -> Result<RetentionSettings, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.set_retention_days(retention_days))
        .await
        .map_err(|error| error.to_string())?
}
