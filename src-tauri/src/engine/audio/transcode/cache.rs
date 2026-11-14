//! Caché de conversión; adapta el diseño de Danilo Wenner a FLAC y SHA-256.
use crate::engine::persist::{config_io::get_data_dir, db::normalize_key};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn dir() -> PathBuf {
    get_data_dir().join("converted-flac-v1")
}
pub fn target_for(original: &str) -> Option<PathBuf> {
    target_in(&dir(), original)
}
pub(super) fn target_in(dir: &Path, original: &str) -> Option<PathBuf> {
    let metadata = std::fs::metadata(original).ok()?;
    if !metadata.is_file() || metadata.len() == 0 {
        return None;
    }
    let stamp = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let name = format!("{:x}", Sha256::digest(normalize_key(original).as_bytes()));
    Some(dir.join(format!("{name}-{stamp:x}-{:x}.flac", metadata.len())))
}
/// Solo actúa sobre archivos regulares propios. Una conversión reciente se conserva.
pub fn cleanup(keep: &Path) {
    let Ok(entries) = std::fs::read_dir(dir()) else {
        return;
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if !kind.is_file() || path.extension().and_then(|s| s.to_str()) != Some("flac") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            files.push((meta.modified().unwrap_or(UNIX_EPOCH), meta.len(), path));
        }
    }
    files.sort_by_key(|f| f.0);
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    let now = SystemTime::now();
    for (modified, size, path) in files {
        let old =
            now.duration_since(modified).unwrap_or_default() > Duration::from_secs(30 * 86400);
        if path != keep && (old || total > 512 * 1024 * 1024) && std::fs::remove_file(path).is_ok()
        {
            total = total.saturating_sub(size);
        }
    }
}
#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
