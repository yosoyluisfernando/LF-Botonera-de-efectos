use super::file_ops::{copy_synced, remove_if_exists, sibling, write_atomic};
use super::install::install_package;
use super::package::{create_package, inspect_package_data};
use super::types::{PendingRestore, RestorePrepared, RestoreResult, FORMAT_VERSION};
use crate::engine::persist::{config_io, db};
use crate::model::AppConfig;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

const MARKER: &str = "restore-pending.json";
const RESULT: &str = "restore-result.json";
const TRANSACTION_DIR: &str = ".restore-transaction";
const STAGED: &str = "staged.lfbackup";

pub fn prepare_restore(
    selected: &Path,
    current_config: &AppConfig,
) -> Result<RestorePrepared, String> {
    let data_dir = config_io::get_data_dir();
    prepare_restore_at(selected, current_config, &data_dir, &db::db_path())
}

pub(crate) fn prepare_restore_at(
    selected: &Path,
    current_config: &AppConfig,
    data_dir: &Path,
    current_db: &Path,
) -> Result<RestorePrepared, String> {
    let marker_path = data_dir.join(MARKER);
    if marker_path.exists() {
        return Err("backup_restore_pending".into());
    }
    let selected_data = inspect_package_data(selected)?;
    let transaction = data_dir.join(TRANSACTION_DIR);
    fs::create_dir_all(&transaction).map_err(|_| "backup_create_directory_failed")?;
    let staged = transaction.join(STAGED);
    let staged_partial = sibling(&staged, "partial")?;
    remove_if_exists(&staged)?;
    remove_if_exists(&staged_partial)?;
    copy_synced(selected, &staged_partial)?;
    fs::rename(&staged_partial, &staged).map_err(|_| "backup_replace_failed")?;
    let staged_data = inspect_package_data(&staged)?;
    if staged_data.summary.backup_id != selected_data.summary.backup_id {
        return Err("backup_copy_mismatch".into());
    }

    let emergency_dir = data_dir.join("automatic-backups");
    fs::create_dir_all(&emergency_dir).map_err(|_| "backup_create_directory_failed")?;
    let emergency = emergency_dir.join(format!(
        "LF-Botonera-antes-de-restaurar-{}-{}.lfbackup",
        Utc::now().format("%Y-%m-%d_%H%M%S"),
        std::process::id()
    ));
    create_package(current_config, current_db, &emergency)?;
    let pending = PendingRestore {
        format_version: FORMAT_VERSION,
        expected_backup_id: staged_data.summary.backup_id.clone(),
        source_path: selected.to_string_lossy().into_owned(),
        emergency_backup_path: emergency.to_string_lossy().into_owned(),
    };
    let bytes = serde_json::to_vec_pretty(&pending).map_err(|_| "backup_marker_invalid")?;
    write_atomic(&marker_path, &bytes)?;
    Ok(RestorePrepared {
        summary: staged_data.summary,
        emergency_backup_path: pending.emergency_backup_path,
    })
}

pub fn recover_pending_restore() -> Result<(), String> {
    let data_dir = config_io::get_data_dir();
    recover_pending_restore_at(&data_dir)
}

pub(crate) fn recover_pending_restore_at(data_dir: &Path) -> Result<(), String> {
    let marker_path = data_dir.join(MARKER);
    if !marker_path.exists() {
        return Ok(());
    }
    let raw = fs::read(&marker_path).map_err(|_| "backup_marker_invalid")?;
    let pending: PendingRestore = match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(_) => {
            return finish_recovery(
                data_dir,
                &marker_path,
                RestoreResult {
                    success: false,
                    summary: None,
                    emergency_backup_path: String::new(),
                    error: "backup_marker_invalid".into(),
                },
            );
        }
    };
    if pending.format_version != FORMAT_VERSION {
        return finish_recovery(
            data_dir,
            &marker_path,
            RestoreResult {
                success: false,
                summary: None,
                emergency_backup_path: pending.emergency_backup_path,
                error: "backup_format_incompatible".into(),
            },
        );
    }
    let staged = data_dir.join(TRANSACTION_DIR).join(STAGED);
    let emergency = PathBuf::from(&pending.emergency_backup_path);
    let attempted = install_package(&staged, Some(&pending.expected_backup_id), &data_dir);
    match attempted {
        Ok(mut summary) => {
            summary.path = pending.source_path;
            finish_recovery(
                data_dir,
                &marker_path,
                RestoreResult {
                    success: true,
                    summary: Some(summary),
                    emergency_backup_path: pending.emergency_backup_path,
                    error: String::new(),
                },
            )
        }
        Err(error) => {
            install_package(&emergency, None, &data_dir)
                .map_err(|rollback| format!("{error};{rollback}"))?;
            finish_recovery(
                &data_dir,
                &marker_path,
                RestoreResult {
                    success: false,
                    summary: None,
                    emergency_backup_path: pending.emergency_backup_path,
                    error,
                },
            )
        }
    }
}

pub fn take_restore_result() -> Result<Option<RestoreResult>, String> {
    take_restore_result_at(&config_io::get_data_dir())
}

pub(crate) fn take_restore_result_at(data_dir: &Path) -> Result<Option<RestoreResult>, String> {
    let path = data_dir.join(RESULT);
    if !path.exists() {
        return Ok(None);
    }
    let result = serde_json::from_slice(&fs::read(&path).map_err(|_| "backup_result_invalid")?)
        .map_err(|_| "backup_result_invalid")?;
    remove_if_exists(&path)?;
    Ok(Some(result))
}

fn finish_recovery(
    data_dir: &Path,
    marker_path: &Path,
    result: RestoreResult,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(&result).map_err(|_| "backup_result_invalid")?;
    write_atomic(&data_dir.join(RESULT), &bytes)?;
    remove_if_exists(marker_path)?;
    let staged = data_dir.join(TRANSACTION_DIR).join(STAGED);
    remove_if_exists(&staged)?;
    remove_if_exists(&sibling(&staged, "partial")?)
}
