//! Purga definitiva de raíces retiradas cuyo plazo de seguridad venció.
use crate::engine::library::time::now_epoch;
use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Default, Serialize)]
pub struct PurgeReport {
    pub roots: usize,
    pub catalog_tracks: usize,
    pub deleted_track_meta: usize,
    pub protected_track_meta: usize,
}

pub fn purge_expired(
    conn: &mut Connection,
    protected: &HashSet<String>,
) -> Result<PurgeReport, String> {
    purge_expired_at(conn, protected, now_epoch())
}

pub fn purge_expired_at(
    conn: &mut Connection,
    protected: &HashSet<String>,
    now: i64,
) -> Result<PurgeReport, String> {
    let ids = expired_ids(conn, now)?;
    let mut report = PurgeReport::default();
    for id in ids {
        purge_one(conn, id, protected, &mut report)?;
    }
    Ok(report)
}

fn purge_one(
    conn: &mut Connection,
    root_id: i64,
    protected: &HashSet<String>,
    report: &mut PurgeReport,
) -> Result<(), String> {
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    let paths = {
        let mut statement = transaction
            .prepare("SELECT path_key FROM library_track WHERE root_id=?1")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![root_id], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };
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
    let deletable = paths
        .iter()
        .filter(|path| !protected.contains(path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    report.protected_track_meta += paths.len() - deletable.len();
    report.deleted_track_meta += delete_orphan_tracks(&transaction, &deletable)?;
    report.catalog_tracks += paths.len();
    report.roots += 1;
    transaction.commit().map_err(|error| error.to_string())
}

fn delete_orphan_tracks(conn: &Connection, paths: &[String]) -> Result<usize, String> {
    if paths.is_empty() {
        return Ok(0);
    }
    let placeholders = std::iter::repeat_n("?", paths.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "DELETE FROM track WHERE path IN ({placeholders})
         AND NOT EXISTS(SELECT 1 FROM library_track WHERE path_key=track.path)"
    );
    conn.execute(&sql, params_from_iter(paths.iter()))
        .map_err(|error| error.to_string())
}

fn expired_ids(conn: &Connection, now: i64) -> Result<Vec<i64>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id FROM library_root
             WHERE enabled=0 AND purge_after IS NOT NULL AND purge_after<=?1",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![now], |row| row.get::<_, i64>(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}
