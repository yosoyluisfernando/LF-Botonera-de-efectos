use super::indexer::{self, SyncProgress, SyncReport};
use super::root_store::{self, AddOutcome, LibraryRoot};
use super::search::{self, SearchResult};
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::persist::db;
use rusqlite::{Connection, Row};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct LibraryService {
    database_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryStatus {
    pub roots: usize,
    pub present: usize,
    pub music: usize,
    pub effects: usize,
    pub pending: usize,
    pub failed: usize,
    pub missing: usize,
}

impl LibraryService {
    pub fn open_default() -> Self {
        Self::new(db::db_path())
    }

    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    pub fn list_roots(&self) -> Result<Vec<LibraryRoot>, String> {
        root_store::list(&self.connection()?)
    }

    pub fn add_root(&self, path: &str, collection: &str) -> Result<AddOutcome, String> {
        let collection = LibraryCollection::parse(collection)?;
        root_store::add(&mut self.connection()?, Path::new(path), collection)
    }

    pub fn remove_root(&self, root_id: i64) -> Result<(), String> {
        root_store::remove(&mut self.connection()?, root_id)
    }

    pub fn sync_root<F>(&self, root_id: i64, progress: F) -> Result<SyncReport, String>
    where
        F: FnMut(SyncProgress),
    {
        indexer::sync_root(&mut self.connection()?, root_id, progress)
    }

    pub fn sync_all<F>(&self, mut progress: F) -> Result<Vec<SyncReport>, String>
    where
        F: FnMut(SyncProgress),
    {
        let mut connection = self.connection()?;
        let roots = root_store::list(&connection)?;
        let mut reports = Vec::new();
        for root in roots.into_iter().filter(|root| root.enabled) {
            reports.push(indexer::sync_root(&mut connection, root.id, &mut progress)?);
        }
        Ok(reports)
    }

    pub fn search(
        &self,
        query: &str,
        collection: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        search::search(&self.connection()?, query, collection, limit)
    }

    pub fn status(&self) -> Result<LibraryStatus, String> {
        let connection = self.connection()?;
        connection
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
            .map_err(|error| error.to_string())
    }

    fn connection(&self) -> Result<Connection, String> {
        db::open(Some(&self.database_path))
    }
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
    })
}
