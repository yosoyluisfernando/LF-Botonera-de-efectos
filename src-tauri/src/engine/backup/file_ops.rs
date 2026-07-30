use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn sibling(path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "backup_invalid_destination".to_string())?;
    Ok(path.with_file_name(format!("{name}.{suffix}")))
}

pub(crate) fn sync_file(path: &Path) -> Result<(), String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| "backup_sync_failed".to_string())
}

pub(crate) fn sync_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .ok_or_else(|| "backup_sync_failed".to_string())?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| "backup_sync_failed".to_string())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| "backup_create_directory_failed")?;
    }
    let partial = sibling(path, "partial")?;
    remove_if_exists(&partial)?;
    let result = (|| {
        let mut file = File::create(&partial).map_err(|_| "backup_write_failed")?;
        file.write_all(bytes).map_err(|_| "backup_write_failed")?;
        file.sync_all().map_err(|_| "backup_sync_failed")?;
        replace_with(&partial, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(partial);
    }
    result
}

pub(crate) fn copy_synced(source: &Path, target: &Path) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|_| "backup_create_directory_failed")?;
    }
    fs::copy(source, target).map_err(|_| "backup_copy_failed")?;
    sync_file(target)
}

pub(crate) fn replace_with(new_file: &Path, target: &Path) -> Result<(), String> {
    let old = sibling(target, "restore-old")?;
    if old.exists() && !target.exists() {
        fs::rename(&old, target).map_err(|_| "backup_replace_failed")?;
    } else {
        remove_if_exists(&old)?;
    }
    if target.exists() {
        fs::rename(target, &old).map_err(|_| "backup_replace_failed")?;
    }
    if let Err(_) = fs::rename(new_file, target) {
        if old.exists() {
            let _ = fs::rename(&old, target);
        }
        return Err("backup_replace_failed".into());
    }
    remove_if_exists(&old)?;
    sync_parent(target)
}

pub(crate) fn remove_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("backup_cleanup_failed".into()),
    }
}
