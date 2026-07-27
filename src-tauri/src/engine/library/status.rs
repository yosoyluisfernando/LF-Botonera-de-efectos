//! Resumen consultable del estado persistente y del observador del catálogo.
use rusqlite::{Connection, Row};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LibraryStatus {
    pub roots: usize,
    pub present: usize,
    pub music: usize,
    pub effects: usize,
    pub pending: usize,
    pub failed: usize,
    pub missing: usize,
    pub monitoring: bool,
    pub monitor_error: Option<String>,
}

pub fn read(
    connection: &Connection,
    monitoring: bool,
    monitor_error: Option<String>,
) -> Result<LibraryStatus, String> {
    let mut status = connection
        .query_row(
            "SELECT
             (SELECT COUNT(*) FROM library_root),
             SUM(CASE WHEN present=1 THEN 1 ELSE 0 END),
             SUM(CASE WHEN present=1 AND collection='music' THEN 1 ELSE 0 END),
             SUM(CASE WHEN present=1 AND collection='effects' THEN 1 ELSE 0 END),
             SUM(CASE WHEN present=1 AND metadata_state='pending' THEN 1 ELSE 0 END),
             SUM(CASE WHEN present=1 AND metadata_state='failed' THEN 1 ELSE 0 END),
             SUM(CASE WHEN present=0 THEN 1 ELSE 0 END)
             FROM library_track",
            [],
            map_status,
        )
        .map_err(|error| error.to_string())?;
    status.monitoring = monitoring;
    status.monitor_error = monitor_error;
    Ok(status)
}

fn map_status(row: &Row) -> rusqlite::Result<LibraryStatus> {
    Ok(LibraryStatus {
        roots: row.get::<_, i64>(0)? as usize,
        present: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as usize,
        music: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as usize,
        effects: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as usize,
        pending: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as usize,
        failed: row.get::<_, Option<i64>>(5)?.unwrap_or(0) as usize,
        missing: row.get::<_, Option<i64>>(6)?.unwrap_or(0) as usize,
        monitoring: false,
        monitor_error: None,
    })
}
