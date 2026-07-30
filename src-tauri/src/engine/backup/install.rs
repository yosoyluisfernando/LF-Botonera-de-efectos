use super::file_ops::{copy_synced, remove_if_exists, replace_with, sibling, write_atomic};
use super::package::inspect_package_data;
use super::types::BackupSummary;
use crate::model::AppConfig;
use std::fs;
use std::path::Path;

pub(crate) fn install_package(
    package: &Path,
    expected_id: Option<&str>,
    data_dir: &Path,
) -> Result<BackupSummary, String> {
    let data = inspect_package_data(package)?;
    if expected_id.is_some_and(|id| id != data.summary.backup_id) {
        return Err("backup_copy_mismatch".into());
    }
    let db_target = data_dir.join("tracks.db");
    if installed_matches(
        &db_target,
        &data.summary.backup_id,
        &data.config_json,
        data_dir,
    ) {
        return Ok(data.summary);
    }
    remove_sidecars(data_dir)?;
    let db_new = sibling(&db_target, "restore-new")?;
    remove_if_exists(&db_new)?;
    copy_synced(package, &db_new)?;
    replace_with(&db_new, &db_target)?;
    write_atomic(
        &data_dir.join("botonera_config.json"),
        data.config_json.as_bytes(),
    )?;
    let installed = inspect_package_data(&db_target)?;
    if installed.summary.backup_id != data.summary.backup_id {
        return Err("backup_restore_verification_failed".into());
    }
    let config: AppConfig = serde_json::from_slice(
        &fs::read(data_dir.join("botonera_config.json"))
            .map_err(|_| "backup_restore_verification_failed")?,
    )
    .map_err(|_| "backup_restore_verification_failed")?;
    if config.profiles.len() as u64 != data.summary.profile_count {
        return Err("backup_restore_verification_failed".into());
    }
    Ok(data.summary)
}

fn installed_matches(db_path: &Path, id: &str, config_json: &str, data_dir: &Path) -> bool {
    let Ok(installed) = inspect_package_data(db_path) else {
        return false;
    };
    if installed.summary.backup_id != id {
        return false;
    }
    fs::read_to_string(data_dir.join("botonera_config.json"))
        .map(|value| value == config_json)
        .unwrap_or(false)
}

fn remove_sidecars(data_dir: &Path) -> Result<(), String> {
    remove_if_exists(&data_dir.join("tracks.db-wal"))?;
    remove_if_exists(&data_dir.join("tracks.db-shm"))
}
