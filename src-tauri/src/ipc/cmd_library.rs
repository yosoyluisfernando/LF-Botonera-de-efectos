use crate::engine::library::browse::{BrowseCursor, BrowsePage};
use crate::engine::library::indexer::{SyncProgress, SyncReport};
use crate::engine::library::root_store::{AddOutcome, LibraryRoot};
use crate::engine::library::search::SearchResult;
use crate::engine::library::service::LibraryStatus;
use crate::ipc::AppState;
use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Deserialize)]
pub struct LibraryRootRequest {
    pub path: String,
    pub collection: String,
}

#[tauri::command]
pub async fn library_list_roots(state: State<'_, AppState>) -> Result<Vec<LibraryRoot>, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.list_roots())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_add_root(
    path: String,
    collection: String,
    state: State<'_, AppState>,
) -> Result<AddOutcome, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.add_root(&path, &collection))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_add_roots(
    roots: Vec<LibraryRootRequest>,
    state: State<'_, AppState>,
) -> Result<Vec<AddOutcome>, String> {
    let requests = roots
        .into_iter()
        .map(|root| (root.path, root.collection))
        .collect::<Vec<_>>();
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.add_roots(&requests))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_remove_root(root_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.remove_root(root_id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_sync_root(
    root_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SyncReport, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.sync_root(root_id, |progress| emit_progress(&app, progress))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_sync_all(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<SyncReport>, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.sync_all(|progress| emit_progress(&app, progress))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_search(
    query: String,
    collection: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.search(&query, collection.as_deref(), limit.unwrap_or(100))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_browse(
    collection: Option<String>,
    limit: Option<usize>,
    cursor: Option<BrowseCursor>,
    direction: Option<String>,
    state: State<'_, AppState>,
) -> Result<BrowsePage, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.browse(
            collection.as_deref(),
            limit.unwrap_or(100),
            cursor.as_ref(),
            direction.as_deref().unwrap_or("forward"),
        )
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_status(state: State<'_, AppState>) -> Result<LibraryStatus, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || service.status())
        .await
        .map_err(|error| error.to_string())?
}

fn emit_progress(app: &AppHandle, progress: SyncProgress) {
    let _ = app.emit("library-index-progress", progress);
}
