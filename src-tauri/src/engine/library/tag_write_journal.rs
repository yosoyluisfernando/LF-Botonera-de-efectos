//! Recuperación de un reemplazo de audio durante escritura de etiquetas.
use crate::engine::persist::{atomic_file, config_io};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = ".tag-write-transaction.json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagWriteJournal {
    pub source: PathBuf,
    pub staged: PathBuf,
    pub backup: PathBuf,
    #[serde(default)]
    pub committed: bool,
}

pub fn store_at(data_dir: &Path, value: &TagWriteJournal) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "library_metadata_write_failed")?;
    atomic_file::write(&path_at(data_dir), &bytes)
}

pub fn clear_at(data_dir: &Path) -> Result<(), String> {
    atomic_file::remove(&path_at(data_dir))
}

pub fn recover() -> Result<(), String> {
    recover_at(&config_io::get_data_dir())
}

pub fn recover_at(data_dir: &Path) -> Result<(), String> {
    let marker = path_at(data_dir);
    if !marker.exists() {
        return Ok(());
    }
    let raw = fs::read(&marker).map_err(|_| "library_metadata_write_failed")?;
    let value: TagWriteJournal =
        serde_json::from_slice(&raw).map_err(|_| "library_metadata_write_failed")?;
    if value.committed {
        remove_if_exists(&value.backup)?;
    } else if value.backup.exists() {
        if value.source.exists() {
            fs::remove_file(&value.source).map_err(|_| "library_metadata_write_failed")?;
        }
        fs::rename(&value.backup, &value.source).map_err(|_| "library_metadata_write_failed")?;
    }
    remove_if_exists(&value.staged)?;
    atomic_file::remove(&marker)
}

pub fn remove_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("library_metadata_write_failed".into()),
    }
}

fn path_at(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE_NAME)
}
