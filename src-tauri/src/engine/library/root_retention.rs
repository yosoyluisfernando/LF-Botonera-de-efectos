//! Retiro reversible de raíces y purga posterior con plazo de seguridad.
pub use super::root_purge::{purge_expired, purge_expired_at, PurgeReport};
use crate::engine::library::time::now_epoch;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;

pub const MIN_RETENTION_DAYS: u16 = 30;
pub const MAX_RETENTION_DAYS: u16 = 365;
const DAY_SECONDS: i64 = 86_400;

#[derive(Clone, Debug, Serialize)]
pub struct RetentionSettings {
    pub retention_days: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct RetiredRoot {
    pub id: i64,
    pub path: String,
    pub path_key: String,
    pub collection: String,
    pub retired_at: i64,
    pub purge_after: i64,
}

pub fn settings(conn: &Connection) -> Result<RetentionSettings, String> {
    conn.query_row(
        "SELECT retention_days FROM library_setting WHERE id=1",
        [],
        |row| {
            Ok(RetentionSettings {
                retention_days: row.get(0)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub fn set_retention_days(conn: &Connection, days: u16) -> Result<RetentionSettings, String> {
    if !(MIN_RETENTION_DAYS..=MAX_RETENTION_DAYS).contains(&days) {
        return Err("library_retention_days_out_of_range".into());
    }
    conn.execute(
        "UPDATE library_setting SET retention_days=?1 WHERE id=1",
        params![days],
    )
    .map_err(|error| error.to_string())?;
    settings(conn)
}

pub fn retire(conn: &Connection, root_id: i64) -> Result<RetiredRoot, String> {
    retire_at(conn, root_id, now_epoch())
}

pub fn retire_at(conn: &Connection, root_id: i64, now: i64) -> Result<RetiredRoot, String> {
    let days = settings(conn)?.retention_days as i64;
    let purge_after = now.saturating_add(days * DAY_SECONDS);
    let changed = conn
        .execute(
            "UPDATE library_root SET enabled=0,state='retired',
             retired_at=?2,purge_after=?3
             WHERE id=?1 AND enabled=1",
            params![root_id, now, purge_after],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("library_root_not_active".into());
    }
    get_retired(conn, root_id)?.ok_or_else(|| "library_root_not_found".into())
}

pub fn restore(conn: &Connection, root_id: i64) -> Result<(), String> {
    let changed = conn
        .execute(
            "UPDATE library_root SET enabled=1,state='pending',
             retired_at=NULL,purge_after=NULL
             WHERE id=?1 AND enabled=0 AND retired_at IS NOT NULL",
            params![root_id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("library_retired_root_not_found".into());
    }
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<RetiredRoot>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id,path,path_key,collection,retired_at,purge_after
             FROM library_root WHERE enabled=0 AND retired_at IS NOT NULL
             ORDER BY purge_after,path_key",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], map_retired)
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn overlapping(conn: &Connection, path_key: &str) -> Result<Option<RetiredRoot>, String> {
    Ok(list(conn)?.into_iter().find(|root| {
        let existing = Path::new(&root.path_key);
        let requested = Path::new(path_key);
        existing.starts_with(requested) || requested.starts_with(existing)
    }))
}

fn get_retired(conn: &Connection, id: i64) -> Result<Option<RetiredRoot>, String> {
    conn.query_row(
        "SELECT id,path,path_key,collection,retired_at,purge_after
         FROM library_root WHERE id=?1 AND enabled=0",
        params![id],
        map_retired,
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn map_retired(row: &rusqlite::Row) -> rusqlite::Result<RetiredRoot> {
    Ok(RetiredRoot {
        id: row.get(0)?,
        path: row.get(1)?,
        path_key: row.get(2)?,
        collection: row.get(3)?,
        retired_at: row.get(4)?,
        purge_after: row.get(5)?,
    })
}

use rusqlite::OptionalExtension;

#[cfg(test)]
#[path = "root_retention_tests.rs"]
mod tests;
