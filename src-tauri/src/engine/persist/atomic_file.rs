//! Reemplazo recuperable de archivos pequeños de estado.
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| "state_directory_failed")?;
    }
    let partial = sibling(path, "partial")?;
    let previous = sibling(path, "previous")?;
    recover_previous(path, &previous)?;
    remove(&partial)?;
    remove(&previous)?;
    let result = (|| {
        let mut file = File::create(&partial).map_err(|_| "state_write_failed")?;
        file.write_all(bytes).map_err(|_| "state_write_failed")?;
        file.sync_all().map_err(|_| "state_sync_failed")?;
        if path.exists() {
            fs::rename(path, &previous).map_err(|_| "state_replace_failed")?;
        }
        if fs::rename(&partial, path).is_err() {
            if previous.exists() {
                let _ = fs::rename(&previous, path);
            }
            return Err("state_replace_failed".into());
        }
        remove(&previous)?;
        sync_parent(path)
    })();
    if result.is_err() {
        let _ = remove(&partial);
    }
    result
}

/// Repara un reemplazo interrumpido antes de intentar leer el estado.
pub fn recover(path: &Path) -> Result<(), String> {
    recover_previous(path, &sibling(path, "previous")?)
}

pub fn remove(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("state_cleanup_failed".into()),
    }
}

fn recover_previous(path: &Path, previous: &Path) -> Result<(), String> {
    if !path.exists() && previous.exists() {
        fs::rename(previous, path).map_err(|_| "state_recovery_failed")?;
    }
    Ok(())
}

fn sibling(path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "state_invalid_path".to_string())?;
    Ok(path.with_file_name(format!("{name}.{suffix}")))
}

fn sync_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let parent = path.parent().ok_or("state_sync_failed")?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| "state_sync_failed".into())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn replaces_existing_state_without_leaving_sidecars() {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("lf-atomic-state-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");
        fs::write(&path, b"old").unwrap();

        write(&path, b"new").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert!(!sibling(&path, "partial").unwrap().exists());
        assert!(!sibling(&path, "previous").unwrap().exists());
        let _ = fs::remove_dir_all(dir);
    }
}
