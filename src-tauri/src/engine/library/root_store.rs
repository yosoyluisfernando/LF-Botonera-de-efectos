use crate::domain::library::root_plan::{plan_root_add, LibraryCollection, RootAddKind, RootSpec};
use crate::engine::library::time::now_epoch;
use crate::engine::library::{root_merge, root_path, root_retention};
use rusqlite::{params, Connection, Transaction};
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
    Restored { root_id: i64 },
}

pub fn list(conn: &Connection) -> Result<Vec<LibraryRoot>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id,path,path_key,collection,enabled,state,scan_generation,last_scan_at
             FROM library_root WHERE enabled=1 ORDER BY collection,path_key",
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
    add_batch(conn, &[(path.to_path_buf(), collection)])?
        .into_iter()
        .next()
        .ok_or_else(|| "library_root_batch_empty".into())
}

pub fn add_batch(
    conn: &mut Connection,
    roots: &[(PathBuf, LibraryCollection)],
) -> Result<Vec<AddOutcome>, String> {
    let normalized = roots
        .iter()
        .map(|(path, collection)| {
            root_path::normalize_root(path).map(|normalized| (normalized, *collection))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    let outcomes = normalized
        .into_iter()
        .map(|((display, key), collection)| add_normalized(&transaction, display, key, collection))
        .collect::<Result<Vec<_>, _>>()?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(outcomes)
}

fn add_normalized(
    transaction: &Transaction<'_>,
    display_path: String,
    path_key: String,
    collection: LibraryCollection,
) -> Result<AddOutcome, String> {
    if let Some(retired) = root_retention::overlapping(transaction, &path_key)? {
        if retired.path_key != path_key {
            return Err("library_root_overlaps_retired".into());
        }
        transaction
            .execute(
                "UPDATE library_root SET enabled=1,state='pending',collection=?2,
                 retired_at=NULL,purge_after=NULL WHERE id=?1",
                params![retired.id, collection.as_str()],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "UPDATE library_track SET collection=?2 WHERE root_id=?1",
                params![retired.id, collection.as_str()],
            )
            .map_err(|error| error.to_string())?;
        return Ok(AddOutcome::Restored {
            root_id: retired.id,
        });
    }
    let plan = plan_root_add(&specs(transaction)?, &path_key, collection);
    match plan.kind {
        RootAddKind::AlreadyCovered { root_id } => {
            return Ok(AddOutcome::AlreadyCovered { root_id });
        }
        RootAddKind::ChangeCollection { root_id } => {
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
            return Ok(AddOutcome::Reclassified { root_id });
        }
        RootAddKind::Add => {}
    }
    transaction
        .execute(
            "INSERT INTO library_root(path,path_key,collection,created_at)
             VALUES(?1,?2,?3,?4)",
            params![display_path, path_key, collection.as_str(), now_epoch()],
        )
        .map_err(|error| error.to_string())?;
    let root_id = transaction.last_insert_rowid();
    root_merge::merge_children(
        transaction,
        root_id,
        &display_path,
        collection,
        &plan.merge_root_ids,
    )?;
    Ok(AddOutcome::Added {
        root_id,
        merged: plan.merge_root_ids,
    })
}

pub fn remove(conn: &mut Connection, root_id: i64) -> Result<root_retention::RetiredRoot, String> {
    root_retention::retire(conn, root_id)
}

pub fn restore(conn: &mut Connection, root_id: i64) -> Result<(), String> {
    let retired = root_retention::list(conn)?
        .into_iter()
        .find(|root| root.id == root_id)
        .ok_or("library_retired_root_not_found")?;
    let collection = LibraryCollection::parse(&retired.collection)?;
    let plan = plan_root_add(&specs(conn)?, &retired.path_key, collection);
    if !matches!(plan.kind, RootAddKind::Add) || !plan.merge_root_ids.is_empty() {
        return Err("library_restore_root_overlap".into());
    }
    root_retention::restore(conn, root_id)
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

#[cfg(test)]
#[path = "root_store_tests.rs"]
mod tests;
