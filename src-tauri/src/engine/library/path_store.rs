//! Migración transaccional de la clave de una pista después de renombrarla.
use crate::engine::persist::db;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::path::{Path, PathBuf};

pub fn move_path(
    connection: &mut Connection,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    let old_key = db::normalize_key(old_path);
    let new_key = db::normalize_key(new_path);
    let new_name = Path::new(new_path)
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("library_rename_invalid_name")?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    if old_key != new_key {
        move_track_key(&transaction, &old_key, &new_key)?;
    }
    update_catalog_name(&transaction, &old_key, &new_key, new_name)?;
    super::search_index::rebuild(&transaction, &new_key)?;
    transaction
        .execute(
            "DELETE FROM library_track_search WHERE path_key=?1 AND ?1<>?2",
            params![old_key, new_key],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

fn move_track_key(
    transaction: &Transaction<'_>,
    old_key: &str,
    new_key: &str,
) -> Result<(), String> {
    let old_exists = exists(transaction, old_key)?;
    let new_exists = exists(transaction, new_key)?;
    match (old_exists, new_exists) {
        (false, true) => return Ok(()),
        (false, false) => return Err("library_track_not_found".into()),
        (true, true) => return Err("library_rename_destination_exists".into()),
        (true, false) => {}
    }
    transaction
        .execute(
            "INSERT INTO track
             SELECT ?2,mtime,size,duration_s,sample_rate,channels,cue_start_s,cue_end_s,
                    gain_db,norm_enabled,norm_gain_db,measured_peak_db,measured_lufs,
                    analyzed_at,last_played
             FROM track WHERE path=?1",
            params![old_key, new_key],
        )
        .map_err(|error| error.to_string())?;
    for table in ["track_user_metadata", "track_keyword", "library_track"] {
        let sql = format!("UPDATE {table} SET path_key=?2 WHERE path_key=?1");
        transaction
            .execute(&sql, params![old_key, new_key])
            .map_err(|error| error.to_string())?;
    }
    transaction
        .execute("DELETE FROM track WHERE path=?1", params![old_key])
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn update_catalog_name(
    transaction: &Transaction<'_>,
    old_key: &str,
    new_key: &str,
    new_name: &str,
) -> Result<(), String> {
    let relative: String = transaction
        .query_row(
            "SELECT relative_path FROM library_track WHERE path_key IN (?1,?2)
             ORDER BY path_key=?2 DESC LIMIT 1",
            params![old_key, new_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "library_track_not_found".to_string())?;
    let next_relative = PathBuf::from(relative)
        .with_file_name(new_name)
        .to_string_lossy()
        .to_string();
    transaction
        .execute(
            "UPDATE library_track SET relative_path=?2,file_name=?3 WHERE path_key=?1",
            params![new_key, next_relative, new_name],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn exists(transaction: &Transaction<'_>, key: &str) -> Result<bool, String> {
    transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM track WHERE path=?1)",
            params![key],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}
