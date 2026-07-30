use super::indexer::{self, SyncProgress, SyncReport};
use super::monitor::{self, LibraryMonitor};
use super::root_store::{self, AddOutcome, LibraryRoot};
use super::search::{self, SearchResult};
use super::status;
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::persist::db;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LibraryService {
    database_path: PathBuf,
    pub(super) operation: Arc<Mutex<()>>,
    monitor: Arc<Mutex<Option<LibraryMonitor>>>,
    monitor_error: Arc<Mutex<Option<String>>>,
}

pub use super::status::LibraryStatus;

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

    pub fn add_roots(&self, roots: &[(String, String)]) -> Result<Vec<AddOutcome>, String> {
        let requests = roots
            .iter()
            .map(|(path, collection)| {
                Ok((PathBuf::from(path), LibraryCollection::parse(collection)?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let outcomes = {
            let _guard = self
                .operation
                .lock()
                .map_err(|_| "library_operation_lock")?;
            root_store::add_batch(&mut self.connection()?, &requests)?
        };
        self.refresh_monitor();
        Ok(outcomes)
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
        let root_ids = root_store::list(&connection)?
            .into_iter()
            .filter(|root| root.enabled)
            .map(|root| root.id)
            .collect::<Vec<_>>();
        indexer::sync_roots(&mut connection, &root_ids, &mut progress)
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
        let monitoring = self
            .monitor
            .lock()
            .map(|monitor| monitor.is_some())
            .unwrap_or(false);
        let monitor_error = self
            .monitor_error
            .lock()
            .ok()
            .and_then(|error| error.clone());
        status::read(&self.connection()?, monitoring, monitor_error)
    }

    pub(super) fn connection(&self) -> Result<Connection, String> {
        db::open(Some(&self.database_path))
    }

    pub(super) fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub(super) fn refresh_monitor(&self) {
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
