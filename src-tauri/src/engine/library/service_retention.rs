//! Operaciones de retención serializadas por el servicio de Biblioteca.
use super::root_retention::{self, PurgeReport, RetentionSettings, RetiredRoot};
use super::root_store;
use super::service::LibraryService;
use std::collections::HashSet;

impl LibraryService {
    pub fn remove_root(&self, root_id: i64) -> Result<RetiredRoot, String> {
        let retired = {
            let _guard = self
                .operation
                .lock()
                .map_err(|_| "library_operation_lock")?;
            root_store::remove(&mut self.connection()?, root_id)?
        };
        self.refresh_monitor();
        Ok(retired)
    }

    pub fn restore_root(&self, root_id: i64) -> Result<(), String> {
        {
            let _guard = self
                .operation
                .lock()
                .map_err(|_| "library_operation_lock")?;
            root_store::restore(&mut self.connection()?, root_id)?;
        }
        self.refresh_monitor();
        Ok(())
    }

    pub fn list_retired_roots(&self) -> Result<Vec<RetiredRoot>, String> {
        root_retention::list(&self.connection()?)
    }

    pub fn retention_settings(&self) -> Result<RetentionSettings, String> {
        root_retention::settings(&self.connection()?)
    }

    pub fn set_retention_days(&self, days: u16) -> Result<RetentionSettings, String> {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| "library_operation_lock")?;
        root_retention::set_retention_days(&self.connection()?, days)
    }

    pub fn purge_expired(&self, protected: &HashSet<String>) -> Result<PurgeReport, String> {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| "library_operation_lock")?;
        root_retention::purge_expired(&mut self.connection()?, protected)
    }
}
