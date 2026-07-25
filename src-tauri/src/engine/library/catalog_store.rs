use super::metadata::LibraryMetadata;
use super::search_text;
use crate::engine::library::time::now_epoch;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;

mod sql;
use sql::{apply_metadata, replace_search, upsert_catalog, upsert_track_placeholder};

#[derive(Debug, Clone)]
pub struct KnownFile {
    pub mtime: i64,
    pub size: i64,
    pub metadata_state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CatalogInput {
    pub path_key: String,
    pub relative_path: String,
    pub file_name: String,
    pub extension: String,
    pub mtime: i64,
    pub size: i64,
    pub content_changed: bool,
    pub needs_metadata: bool,
}

#[derive(Debug)]
pub struct MetadataUpdate {
    pub path_key: String,
    pub result: Result<LibraryMetadata, String>,
}

pub fn known_files(conn: &Connection) -> Result<HashMap<String, KnownFile>, String> {
    let mut statement = conn
        .prepare(
            "SELECT t.path,t.mtime,t.size,lt.metadata_state
             FROM track t LEFT JOIN library_track lt ON lt.path_key=t.path",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                KnownFile {
                    mtime: row.get(1)?,
                    size: row.get(2)?,
                    metadata_state: row.get(3)?,
                },
            ))
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<HashMap<_, _>, _>>()
        .map_err(|error| error.to_string())
}

pub fn begin_scan(conn: &Connection, root_id: i64) -> Result<i64, String> {
    conn.query_row(
        "UPDATE library_root SET state='scanning',scan_generation=scan_generation+1
         WHERE id=?1 AND enabled=1 RETURNING scan_generation",
        params![root_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "library_root_not_enabled".into())
}

pub fn save_discovery(
    conn: &mut Connection,
    root_id: i64,
    collection: &str,
    generation: i64,
    files: &[CatalogInput],
) -> Result<(), String> {
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    for file in files {
        if file.content_changed {
            upsert_track_placeholder(&transaction, file)?;
        }
        upsert_catalog(&transaction, root_id, collection, generation, file)?;
        if file.content_changed || file.metadata_state_is_new() {
            replace_search(
                &transaction,
                &file.path_key,
                &search_text::build([
                    Some(file.file_name.as_str()),
                    Some(file.relative_path.as_str()),
                ]),
            )?;
        }
    }
    transaction.commit().map_err(|error| error.to_string())
}

pub fn save_metadata_batch(
    conn: &mut Connection,
    updates: &[MetadataUpdate],
) -> Result<(), String> {
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    for update in updates {
        apply_metadata(&transaction, update)?;
    }
    transaction.commit().map_err(|error| error.to_string())
}

pub fn mark_root_error(conn: &Connection, root_id: i64) {
    let _ = conn.execute(
        "UPDATE library_root SET state='error' WHERE id=?1",
        params![root_id],
    );
}

pub fn mark_path_missing(conn: &Connection, path_key: &str) -> Result<usize, String> {
    let prefix = format!("{path_key}{}", std::path::MAIN_SEPARATOR);
    conn.execute(
        "UPDATE library_track SET present=0
         WHERE path_key=?1 OR substr(path_key,1,length(?2))=?2",
        params![path_key, prefix],
    )
    .map_err(|error| error.to_string())
}

pub fn finish_scan(conn: &mut Connection, root_id: i64, generation: i64) -> Result<usize, String> {
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    let missing = transaction
        .execute(
            "UPDATE library_track SET present=0
             WHERE root_id=?1 AND scan_generation<>?2 AND present=1",
            params![root_id, generation],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE library_root SET state='ready',last_scan_at=?2 WHERE id=?1",
            params![root_id, now_epoch()],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(missing)
}

impl CatalogInput {
    fn metadata_state_is_new(&self) -> bool {
        self.needs_metadata && !self.content_changed
    }
}
