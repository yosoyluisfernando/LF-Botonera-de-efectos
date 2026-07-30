//! Renombrado físico coordinado con catálogo, configuración y recuperación.
use super::rename_journal::{self, RenameJournal};
use crate::domain::library::path_rewrite;
use crate::engine::persist::{config_io, db};
use crate::model::AppConfig;
use rusqlite::{params, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

pub fn rename_at(
    database_path: &Path,
    data_dir: &Path,
    config: &mut AppConfig,
    old_path: &str,
    new_file_name: &str,
) -> Result<(String, String), String> {
    let source = PathBuf::from(old_path);
    validate_source(database_path, &source)?;
    let target = super::file_name::destination(&source, new_file_name)?;
    let new_path = target.to_string_lossy().to_string();
    let mut next_config = config.clone();
    path_rewrite::replace_in_config(&mut next_config, old_path, &new_path);
    let journal = RenameJournal {
        old_path: old_path.to_string(),
        new_path: new_path.clone(),
        old_config: config.clone(),
        new_config: next_config.clone(),
    };
    rename_journal::store_at(data_dir, &journal)?;
    let attempted = (|| {
        rename_physical(&source, &target)?;
        move_database(database_path, old_path, &new_path)?;
        config_io::save_config_at(&data_dir.join("botonera_config.json"), &next_config)
    })();
    if let Err(error) = attempted {
        let rollback = rollback_at(database_path, data_dir, &journal);
        return match rollback {
            Ok(()) => Err(public_error(error)),
            Err(rollback) => {
                eprintln!(
                    "No se pudo recuperar el renombrado ({error}) ni revertirlo ({rollback})"
                );
                Err("library_rename_recovery_failed".into())
            }
        };
    }
    *config = next_config;
    rename_journal::clear_at(data_dir)?;
    Ok((old_path.to_string(), new_path))
}

fn public_error(error: String) -> String {
    if error.starts_with("library_") {
        error
    } else {
        "library_rename_failed".into()
    }
}

pub fn recover_pending() -> Result<(), String> {
    let data_dir = config_io::get_data_dir();
    recover_pending_at(&db::db_path(), &data_dir)
}

pub fn recover_pending_at(database_path: &Path, data_dir: &Path) -> Result<(), String> {
    let Some(journal) = rename_journal::load_at(data_dir)? else {
        return Ok(());
    };
    let source = Path::new(&journal.old_path);
    let target = Path::new(&journal.new_path);
    let same_key = db::normalize_key(&journal.old_path) == db::normalize_key(&journal.new_path);
    if target.exists() && (same_key || !source.exists()) {
        move_database(database_path, &journal.old_path, &journal.new_path)?;
        config_io::save_config_at(&data_dir.join("botonera_config.json"), &journal.new_config)?;
    } else if source.exists() && !target.exists() {
        move_database(database_path, &journal.new_path, &journal.old_path)?;
        config_io::save_config_at(&data_dir.join("botonera_config.json"), &journal.old_config)?;
    } else {
        return Err("library_rename_recovery_failed".into());
    }
    rename_journal::clear_at(data_dir)
}

fn rollback_at(
    database_path: &Path,
    data_dir: &Path,
    journal: &RenameJournal,
) -> Result<(), String> {
    let source = Path::new(&journal.old_path);
    let target = Path::new(&journal.new_path);
    if target.exists() {
        rename_physical(target, source)?;
    }
    move_database(database_path, &journal.new_path, &journal.old_path)?;
    config_io::save_config_at(&data_dir.join("botonera_config.json"), &journal.old_config)?;
    rename_journal::clear_at(data_dir)
}

fn validate_source(database_path: &Path, source: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source).map_err(|_| "library_rename_invalid_source")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("library_rename_invalid_source".into());
    }
    let connection = db::open(Some(database_path))?;
    let key = db::normalize_key(&source.to_string_lossy());
    let found: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_track lt JOIN library_root lr ON lr.id=lt.root_id
             WHERE lt.path_key=?1 AND lt.present=1 AND lr.enabled=1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    found
        .is_some()
        .then_some(())
        .ok_or_else(|| "library_track_not_found".into())
}

fn move_database(database_path: &Path, old_path: &str, new_path: &str) -> Result<(), String> {
    let mut connection = db::open(Some(database_path))?;
    super::path_store::move_path(&mut connection, old_path, new_path)
}

fn rename_physical(source: &Path, target: &Path) -> Result<(), String> {
    let same_key = db::normalize_key(&source.to_string_lossy())
        == db::normalize_key(&target.to_string_lossy());
    if !same_key {
        return fs::rename(source, target).map_err(|_| "library_rename_locked".into());
    }
    let temporary = super::file_name::temporary_sibling(source)?;
    fs::rename(source, &temporary).map_err(|_| "library_rename_locked")?;
    if fs::rename(&temporary, target).is_err() {
        let _ = fs::rename(&temporary, source);
        return Err("library_rename_failed".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "file_rename_tests.rs"]
mod tests;
