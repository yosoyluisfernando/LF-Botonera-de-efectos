use super::file_ops::{remove_if_exists, sync_file, sync_parent};
use super::types::{BackupSummary, PackageData, FORMAT_VERSION, MANIFEST_TABLE};
use super::validation::{
    count_tracks, ensure_integrity, new_backup_id, palette_count, schema_version,
    validate_manifest, ManifestValues,
};
use crate::model::AppConfig;
use chrono::Utc;
use rusqlite::{backup::Backup, params, Connection, OpenFlags};
use std::fs;
use std::path::Path;
use std::time::Duration;

const CREATE_MANIFEST: &str = "CREATE TABLE lf_backup_manifest (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 format_version INTEGER NOT NULL, backup_id TEXT NOT NULL,
 created_at TEXT NOT NULL, app_version TEXT NOT NULL,
 source_platform TEXT NOT NULL, database_schema INTEGER NOT NULL,
 profile_count INTEGER NOT NULL, palette_count INTEGER NOT NULL,
 track_count INTEGER NOT NULL, config_json TEXT NOT NULL
)";

pub fn create_package(
    config: &AppConfig,
    source_db: &Path,
    destination: &Path,
) -> Result<BackupSummary, String> {
    if destination.exists() {
        return Err("backup_destination_exists".into());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "backup_invalid_destination".to_string())?;
    fs::create_dir_all(parent).map_err(|_| "backup_create_directory_failed")?;
    let partial = destination.with_extension("lfbackup.partial");
    remove_if_exists(&partial)?;
    let result = create_partial(config, source_db, &partial)
        .and_then(|_| fs::rename(&partial, destination).map_err(|_| "backup_replace_failed".into()))
        .and_then(|_| sync_parent(destination))
        .and_then(|_| inspect_package(destination));
    if result.is_err() {
        let _ = fs::remove_file(&partial);
    }
    result
}

fn create_partial(config: &AppConfig, source_db: &Path, path: &Path) -> Result<(), String> {
    let source = Connection::open_with_flags(source_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "backup_database_open_failed")?;
    let mut destination = Connection::open(path).map_err(|_| "backup_write_failed")?;
    {
        let backup =
            Backup::new(&source, &mut destination).map_err(|_| "backup_database_copy_failed")?;
        backup
            .run_to_completion(256, Duration::from_millis(10), None)
            .map_err(|_| "backup_database_copy_failed")?;
    }
    let _: String = destination
        .query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0))
        .map_err(|_| "backup_database_finalize_failed")?;
    destination
        .execute(&format!("DROP TABLE IF EXISTS {MANIFEST_TABLE}"), [])
        .map_err(|_| "backup_database_finalize_failed")?;
    destination
        .execute_batch(CREATE_MANIFEST)
        .map_err(|_| "backup_database_finalize_failed")?;
    let config_json = serde_json::to_string(config).map_err(|_| "backup_config_invalid")?;
    let profiles = config.profiles.len() as u64;
    let palettes = palette_count(config);
    let tracks = count_tracks(&destination)?;
    let schema = schema_version(&destination)?;
    destination
        .execute(
            "INSERT INTO lf_backup_manifest VALUES
             (1,?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                FORMAT_VERSION,
                new_backup_id(),
                Utc::now().to_rfc3339(),
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                schema,
                profiles,
                palettes,
                tracks,
                config_json
            ],
        )
        .map_err(|_| "backup_database_finalize_failed")?;
    ensure_integrity(&destination)?;
    drop(destination);
    sync_file(path)
}

pub fn inspect_package(path: &Path) -> Result<BackupSummary, String> {
    inspect_package_data(path).map(|data| data.summary)
}

pub(crate) fn inspect_package_data(path: &Path) -> Result<PackageData, String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "backup_invalid_file")?;
    ensure_integrity(&connection)?;
    let values: ManifestValues = connection
        .query_row(
            "SELECT format_version,backup_id,created_at,app_version,source_platform,
                    database_schema,profile_count,palette_count,track_count,config_json
             FROM lf_backup_manifest WHERE singleton=1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, u64>(6)?,
                    row.get::<_, u64>(7)?,
                    row.get::<_, u64>(8)?,
                    row.get::<_, String>(9)?,
                ))
            },
        )
        .map_err(|_| "backup_manifest_missing")?;
    validate_manifest(&connection, &values, path)
}
