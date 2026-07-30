//! Respaldo atómico de configuración + `tracks.db` en un único `.lfbackup`.
//! La base se copia con la API online de SQLite; la restauración ocurre antes
//! de abrir `AppState`, nunca sobre conexiones vivas.
mod file_ops;
mod install;
mod package;
mod restore;
mod types;
mod validation;

pub use package::{create_package, inspect_package};
pub use restore::{prepare_restore, recover_pending_restore, take_restore_result};
pub use types::{BackupInspection, BackupSummary, RestorePrepared, RestoreResult};

#[cfg(test)]
mod metadata_tests;
#[cfg(test)]
mod real_copy_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wal_tests;
