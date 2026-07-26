//! Normalización única de rutas del catálogo en Windows y Linux.
use crate::engine::persist::db::normalize_key;
use std::path::{Path, PathBuf};

pub fn normalize_root(path: &Path) -> Result<(String, String), String> {
    if !path.is_dir() {
        return Err("library_root_not_directory".into());
    }
    normalize_path(path)
}

pub fn normalize_path(path: &Path) -> Result<(String, String), String> {
    let canonical = std::fs::canonicalize(path)
        .or_else(|_| {
            let parent = path.parent().ok_or(std::io::ErrorKind::NotFound)?;
            let name = path.file_name().ok_or(std::io::ErrorKind::NotFound)?;
            std::fs::canonicalize(parent).map(|canonical_parent| canonical_parent.join(name))
        })
        .map_err(|_| "library_path_unavailable")?;
    let display = clean_windows_prefix(&canonical);
    let text = display.to_string_lossy().to_string();
    Ok((text.clone(), normalize_key(&text)))
}

fn clean_windows_prefix(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}
