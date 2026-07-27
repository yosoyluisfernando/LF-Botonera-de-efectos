//! Exploración local para la ventana Biblioteca. Nunca indexa ni persiste rutas.
use crate::engine::audio::formats::is_audio_path;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize)]
pub struct StorageRoot {
    pub path: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileSystemEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

pub fn storage_roots() -> Vec<StorageRoot> {
    platform_roots()
        .into_iter()
        .filter(|path| path.is_dir())
        .map(|path| StorageRoot {
            name: storage_name(&path),
            path: path.to_string_lossy().to_string(),
        })
        .collect()
}

pub fn read_directory(path: &Path) -> Result<Vec<FileSystemEntry>, String> {
    if !path.is_dir() {
        return Err("library_directory_unavailable".into());
    }
    let mut entries = fs::read_dir(path)
        .map_err(|_| "library_directory_denied")?
        .filter_map(Result::ok)
        .filter_map(to_entry)
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .is_directory
            .cmp(&left.is_directory)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(entries)
}

fn to_entry(entry: fs::DirEntry) -> Option<FileSystemEntry> {
    let path = entry.path();
    let file_type = entry.file_type().ok()?;
    if !file_type.is_dir() && !(file_type.is_file() && is_audio_path(&path)) {
        return None;
    }
    Some(FileSystemEntry {
        name: entry.file_name().to_string_lossy().to_string(),
        path: path.to_string_lossy().to_string(),
        is_directory: file_type.is_dir(),
    })
}

#[cfg(windows)]
fn platform_roots() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\", letter as char)))
        .collect()
}

#[cfg(not(windows))]
fn platform_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("/")];
    for parent in ["/media", "/mnt", "/run/media"] {
        collect_mount_children(Path::new(parent), &mut roots);
    }
    roots
}

#[cfg(not(windows))]
fn collect_mount_children(parent: &Path, roots: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                roots.push(path);
            }
        }
    }
}

fn storage_name(path: &Path) -> String {
    #[cfg(windows)]
    {
        return format!("Unidad ({})", path.to_string_lossy().trim_end_matches('\\'));
    }
    #[cfg(not(windows))]
    {
        if path == Path::new("/") {
            return "/".into();
        }
        path.file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_directory_is_rejected() {
        assert_eq!(
            read_directory(Path::new("__missing_library_directory__")).unwrap_err(),
            "library_directory_unavailable"
        );
    }
}
