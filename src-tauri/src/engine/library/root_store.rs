use crate::domain::library::root_plan::{plan_root_add, LibraryCollection, RootAddKind, RootSpec};
use crate::engine::library::search_text;
use crate::engine::library::time::now_epoch;
use crate::engine::persist::db::normalize_key;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize)]
pub struct LibraryRoot {
    pub id: i64,
    pub path: String,
    pub path_key: String,
    pub collection: String,
    pub enabled: bool,
    pub state: String,
    pub scan_generation: i64,
    pub last_scan_at: Option<i64>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum AddOutcome {
    Added { root_id: i64, merged: Vec<i64> },
    AlreadyCovered { root_id: i64 },
    Reclassified { root_id: i64 },
}

pub fn list(conn: &Connection) -> Result<Vec<LibraryRoot>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id,path,path_key,collection,enabled,state,scan_generation,last_scan_at
             FROM library_root ORDER BY collection,path_key",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(LibraryRoot {
                id: row.get(0)?,
                path: row.get(1)?,
                path_key: row.get(2)?,
                collection: row.get(3)?,
                enabled: row.get::<_, i64>(4)? != 0,
                state: row.get(5)?,
                scan_generation: row.get(6)?,
                last_scan_at: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<LibraryRoot>, String> {
    list(conn).map(|roots| roots.into_iter().find(|root| root.id == id))
}

pub fn add(
    conn: &mut Connection,
    path: &Path,
    collection: LibraryCollection,
) -> Result<AddOutcome, String> {
    let (display_path, path_key) = normalize_root(path)?;
    let existing = specs(conn)?;
    let plan = plan_root_add(&existing, &path_key, collection);
    match plan.kind {
        RootAddKind::AlreadyCovered { root_id } => {
            return Ok(AddOutcome::AlreadyCovered { root_id });
        }
        RootAddKind::ChangeCollection { root_id } => {
            let transaction = conn.transaction().map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "UPDATE library_root SET collection=?1,state='pending' WHERE id=?2",
                    params![collection.as_str(), root_id],
                )
                .map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "UPDATE library_track SET collection=?1 WHERE root_id=?2",
                    params![collection.as_str(), root_id],
                )
                .map_err(|error| error.to_string())?;
            transaction.commit().map_err(|error| error.to_string())?;
            return Ok(AddOutcome::Reclassified { root_id });
        }
        RootAddKind::Add => {}
    }
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    transaction
        .execute(
            "INSERT INTO library_root(path,path_key,collection,created_at)
             VALUES(?1,?2,?3,?4)",
            params![display_path, path_key, collection.as_str(), now_epoch()],
        )
        .map_err(|error| error.to_string())?;
    let root_id = transaction.last_insert_rowid();
    for merged in &plan.merge_root_ids {
        let child_path = transaction
            .query_row(
                "SELECT path FROM library_root WHERE id=?1",
                params![merged],
                |row| row.get::<_, String>(0),
            )
            .map_err(|error| error.to_string())?;
        let relative = Path::new(&child_path)
            .strip_prefix(Path::new(&display_path))
            .map_err(|_| "library_root_merge_path")?;
        let prefix = format!(
            "{}{}",
            relative.to_string_lossy(),
            std::path::MAIN_SEPARATOR
        );
        let search_prefix = search_text::normalize(&relative.to_string_lossy());
        transaction
            .execute(
                "UPDATE library_track_search SET search_text=?1 || ' ' || search_text
                 WHERE path_key IN (
                   SELECT path_key FROM library_track WHERE root_id=?2
                 )",
                params![search_prefix, merged],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "UPDATE library_track SET root_id=?1,collection=?2,
                 relative_path=?4 || relative_path WHERE root_id=?3",
                params![root_id, collection.as_str(), merged, prefix],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute("DELETE FROM library_root WHERE id=?1", params![merged])
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(AddOutcome::Added {
        root_id,
        merged: plan.merge_root_ids,
    })
}

pub fn remove(conn: &mut Connection, root_id: i64) -> Result<(), String> {
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    let exists = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM library_root WHERE id=?1)",
            params![root_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err("library_root_not_found".into());
    }
    transaction
        .execute(
            "DELETE FROM library_track_search
             WHERE path_key IN (SELECT path_key FROM library_track WHERE root_id=?1)",
            params![root_id],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute("DELETE FROM library_root WHERE id=?1", params![root_id])
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

fn specs(conn: &Connection) -> Result<Vec<RootSpec>, String> {
    list(conn)?
        .into_iter()
        .map(|root| {
            Ok(RootSpec {
                id: root.id,
                path_key: root.path_key,
                collection: LibraryCollection::parse(&root.collection)?,
            })
        })
        .collect()
}

fn normalize_root(path: &Path) -> Result<(String, String), String> {
    if !path.is_dir() {
        return Err("library_root_not_directory".into());
    }
    let canonical = std::fs::canonicalize(path).map_err(|_| "library_root_unavailable")?;
    let display = clean_windows_prefix(&canonical);
    let text = display.to_string_lossy().to_string();
    Ok((text.clone(), normalize_key(&text)))
}

fn clean_windows_prefix(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}

#[cfg(test)]
#[path = "root_store_tests.rs"]
mod tests;
