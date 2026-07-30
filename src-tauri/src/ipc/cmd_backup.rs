use crate::engine::backup::{
    create_package, inspect_package, prepare_restore, take_restore_result, BackupInspection,
    BackupSummary, RestorePrepared, RestoreResult,
};
use crate::engine::persist::{config_io, db, last_played};
use crate::ipc::AppState;
use chrono::Local;
use std::path::PathBuf;
use tauri::{AppHandle, State};

const FILTER: &str = "LF Botonera (.lfbackup)";
const EXTENSION: &str = "lfbackup";

#[tauri::command]
pub fn backup_create(state: State<'_, AppState>) -> Result<Option<BackupSummary>, String> {
    let _operation = state
        .backup_operation
        .lock()
        .map_err(|_| "backup_operation_unavailable")?;
    let default_name = format!(
        "LF-Botonera-respaldo-{}.lfbackup",
        Local::now().format("%Y-%m-%d_%H%M")
    );
    let Some(mut destination) = rfd::FileDialog::new()
        .add_filter(FILTER, &[EXTENSION])
        .set_file_name(&default_name)
        .save_file()
    else {
        return Ok(None);
    };
    if destination.extension().is_none() {
        destination.set_extension(EXTENSION);
    }
    flush_pending(&state);
    let config = state
        .config
        .lock()
        .map_err(|_| "backup_config_unavailable")?
        .clone();
    create_package(&config, &db::db_path(), &destination).map(Some)
}

#[tauri::command]
pub fn backup_choose_restore(
    state: State<'_, AppState>,
) -> Result<Option<BackupInspection>, String> {
    let _operation = state
        .backup_operation
        .lock()
        .map_err(|_| "backup_operation_unavailable")?;
    let Some(path) = rfd::FileDialog::new()
        .add_filter(FILTER, &[EXTENSION])
        .pick_file()
    else {
        return Ok(None);
    };
    let data = inspect_package(&path)?;
    Ok(Some(BackupInspection {
        summary: data,
        source_path: path.to_string_lossy().into_owned(),
    }))
}

#[tauri::command]
pub fn backup_prepare_restore(
    source_path: String,
    state: State<'_, AppState>,
) -> Result<RestorePrepared, String> {
    let _operation = state
        .backup_operation
        .lock()
        .map_err(|_| "backup_operation_unavailable")?;
    flush_pending(&state);
    let config = state
        .config
        .lock()
        .map_err(|_| "backup_config_unavailable")?
        .clone();
    prepare_restore(&PathBuf::from(source_path), &config)
}

#[tauri::command]
pub fn backup_restart(app: AppHandle, state: State<'_, AppState>) {
    flush_pending(&state);
    app.restart();
}

#[tauri::command]
pub fn backup_take_restore_result() -> Result<Option<RestoreResult>, String> {
    take_restore_result()
}

fn flush_pending(state: &State<'_, AppState>) {
    last_played::flush_now(&state.last_played.handle(), &state.tracks);
    let _ = config_io::save_config(
        &state
            .config
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
    );
}
