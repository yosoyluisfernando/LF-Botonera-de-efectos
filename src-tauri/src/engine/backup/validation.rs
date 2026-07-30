use super::types::{BackupSummary, PackageData, FORMAT_VERSION};
use crate::engine::persist::db;
use crate::model::AppConfig;
use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

pub(crate) type ManifestValues = (
    i64,
    String,
    String,
    String,
    String,
    i64,
    u64,
    u64,
    u64,
    String,
);

pub(crate) fn validate_manifest(
    connection: &Connection,
    values: &ManifestValues,
    path: &Path,
) -> Result<PackageData, String> {
    if values.0 != FORMAT_VERSION {
        return Err("backup_format_incompatible".into());
    }
    if values.5 > db::SCHEMA_VERSION {
        return Err("backup_database_newer".into());
    }
    let config: AppConfig = serde_json::from_str(&values.9).map_err(|_| "backup_config_invalid")?;
    let profiles = config.profiles.len() as u64;
    let palettes = palette_count(&config);
    if config.profiles.is_empty()
        || profiles != values.6
        || palettes != values.7
        || count_tracks(connection)? != values.8
    {
        return Err("backup_manifest_mismatch".into());
    }
    let file_size = fs::metadata(path).map_err(|_| "backup_invalid_file")?.len();
    Ok(PackageData {
        summary: BackupSummary {
            backup_id: values.1.clone(),
            created_at: values.2.clone(),
            app_version: values.3.clone(),
            source_platform: values.4.clone(),
            format_version: values.0,
            database_schema: values.5,
            profile_count: values.6,
            palette_count: values.7,
            track_count: values.8,
            file_size,
            integrity_ok: true,
            audio_included: false,
            path: path.to_string_lossy().into_owned(),
        },
        config_json: values.9.clone(),
    })
}

pub(crate) fn palette_count(config: &AppConfig) -> u64 {
    config
        .profiles
        .iter()
        .map(|item| item.paletas.len() as u64)
        .sum()
}

pub(crate) fn count_tracks(connection: &Connection) -> Result<u64, String> {
    connection
        .query_row("SELECT COUNT(*) FROM track", [], |row| row.get(0))
        .map_err(|_| "backup_database_invalid".into())
}

pub(crate) fn schema_version(connection: &Connection) -> Result<i64, String> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "backup_database_invalid".into())
}

pub(crate) fn ensure_integrity(connection: &Connection) -> Result<(), String> {
    let result: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| "backup_integrity_failed")?;
    (result == "ok")
        .then_some(())
        .ok_or_else(|| "backup_integrity_failed".into())
}

pub(crate) fn new_backup_id() -> String {
    format!("{}-{}", Utc::now().timestamp_micros(), std::process::id())
}
