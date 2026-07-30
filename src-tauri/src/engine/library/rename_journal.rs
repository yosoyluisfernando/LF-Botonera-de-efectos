//! Journal duradero para completar un renombrado después de una interrupción.
use crate::engine::persist::atomic_file;
use crate::model::AppConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = ".rename-transaction.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameJournal {
    pub old_path: String,
    pub new_path: String,
    pub old_config: AppConfig,
    pub new_config: AppConfig,
}

pub fn store_at(data_dir: &Path, journal: &RenameJournal) -> Result<(), String> {
    let bytes = serde_json::to_vec(journal).map_err(|_| "library_rename_journal_invalid")?;
    atomic_file::write(&path_at(data_dir), &bytes)
}

pub fn load_at(data_dir: &Path) -> Result<Option<RenameJournal>, String> {
    let marker = path_at(data_dir);
    if !marker.exists() {
        return Ok(None);
    }
    let bytes = fs::read(marker).map_err(|_| "library_rename_journal_invalid")?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| "library_rename_journal_invalid".into())
}

pub fn clear_at(data_dir: &Path) -> Result<(), String> {
    atomic_file::remove(&path_at(data_dir))
}

fn path_at(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE_NAME)
}
