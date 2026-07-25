use super::indexer::{self, SyncProgress, SyncReport};
use super::monitor::{self, LibraryMonitor};
use super::root_store::{self, AddOutcome, LibraryRoot};
use super::search::{self, SearchResult};
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::persist::db;
use rusqlite::{Connection, Row};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LibraryService {
    database_path: PathBuf,
    operation: Arc<Mutex<()>>,
    monitor: Arc<Mutex<Option<LibraryMonitor>>>,
    monitor_error: Arc<Mutex<Option<String>>>,
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
    pub monitoring: bool,
    pub monitor_error: Option<String>,
}

impl LibraryService {
    pub fn open_default() -> Self {
        Self::new(db::db_path())
    }

    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            operation: Arc::new(Mutex::new(())),
            monitor: Arc::new(Mutex::new(None)),
            monitor_error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_monitoring(&self) -> Result<(), String> {
        let roots = self.list_roots()?;
        let started = monitor::start(
            self.database_path.clone(),
            Arc::clone(&self.operation),
            &roots,
        );
        let (watcher, warning) = match started {
            Ok(value) => value,
            Err(error) => {
                self.set_monitor_error(Some(error.clone()));
                return Err(error);
            }
        };
        *self.monitor.lock().map_err(|_| "library_monitor_lock")? = Some(watcher);
        self.set_monitor_error(warning);
        Ok(())
    }

    pub fn list_roots(&self) -> Result<Vec<LibraryRoot>, String> {
        root_store::list(&self.connection()?)
    }

    pub fn add_root(&self, path: &str, collection: &str) -> Result<AddOutcome, String> {
        let collection = LibraryCollection::parse(collection)?;
        let outcome = {
            let _guard = self
                .operation
                .lock()
                .map_err(|_| "library_operation_lock")?;
            root_store::add(&mut self.connection()?, Path::new(path), collection)?
        };
        self.refresh_monitor();
        Ok(outcome)
    }

    pub fn remove_root(&self, root_id: i64) -> Result<(), String> {
        {
            let _guard = self
                .operation
                .lock()
                .map_err(|_| "library_operation_lock")?;
            root_store::remove(&mut self.connection()?, root_id)?;
        }
        self.refresh_monitor();
        Ok(())
    }

    pub fn sync_root<F>(&self, root_id: i64, progress: F) -> Result<SyncReport, String>
    where
        F: FnMut(SyncProgress),
    {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| "library_operation_lock")?;
        indexer::sync_root(&mut self.connection()?, root_id, progress)
    }

    pub fn sync_all<F>(&self, mut progress: F) -> Result<Vec<SyncReport>, String>
    where
        F: FnMut(SyncProgress),
    {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| "library_operation_lock")?;
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
        status.monitoring = self
            .monitor
            .lock()
            .map(|monitor| monitor.is_some())
            .unwrap_or(false);
        status.monitor_error = self
            .monitor_error
            .lock()
            .ok()
            .and_then(|error| error.clone());
        Ok(status)
    }

    fn connection(&self) -> Result<Connection, String> {
        db::open(Some(&self.database_path))
    }

    fn refresh_monitor(&self) {
        let result = self.list_roots().and_then(|roots| {
            let mut monitor = self.monitor.lock().map_err(|_| "library_monitor_lock")?;
            if let Some(monitor) = monitor.as_mut() {
                monitor.refresh(&roots)?;
            }
            Ok(())
        });
        self.set_monitor_error(result.err());
    }

    fn set_monitor_error(&self, error: Option<String>) {
        if let Ok(mut target) = self.monitor_error.lock() {
            *target = error;
        }
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
        monitoring: false,
        monitor_error: None,
    })
}
