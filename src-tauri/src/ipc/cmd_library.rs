use crate::engine::library::browse::{BrowseCursor, BrowsePage};
use crate::engine::library::browse_window::BrowseWindow;
use crate::engine::library::catalog_tree::CatalogFolder;
use crate::engine::library::filesystem::{self, FileSystemEntry, StorageRoot};
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
    root_id: Option<i64>,
    relative_prefix: Option<String>,
    limit: Option<usize>,
    cursor: Option<BrowseCursor>,
    direction: Option<String>,
    state: State<'_, AppState>,
) -> Result<BrowsePage, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.browse(
            collection.as_deref(),
            root_id,
            relative_prefix.as_deref(),
            limit.unwrap_or(100),
            cursor.as_ref(),
            direction.as_deref().unwrap_or("forward"),
        )
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_folder_children(
    root_id: i64,
    relative_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<CatalogFolder>, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.folder_children(root_id, relative_path.as_deref())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn library_browse_window(
    collection: Option<String>,
    root_id: Option<i64>,
    relative_prefix: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<BrowseWindow, String> {
    let service = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.browse_window(
            collection.as_deref(),
            root_id,
            relative_prefix.as_deref(),
            offset.unwrap_or(0),
            limit.unwrap_or(180),
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

#[tauri::command]
pub async fn library_storage_roots() -> Result<Vec<StorageRoot>, String> {
    tauri::async_runtime::spawn_blocking(filesystem::storage_roots)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn library_read_directory(path: String) -> Result<Vec<FileSystemEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        filesystem::read_directory(std::path::Path::new(&path))
    })
    .await
    .map_err(|error| error.to_string())?
}

fn emit_progress(app: &AppHandle, progress: SyncProgress) {
    let _ = app.emit("library-index-progress", progress);
}
