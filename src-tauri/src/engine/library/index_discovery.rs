//! Primera fase: descubre y guarda nombres de una raíz sin abrir el audio.
use super::catalog_store::{self, CatalogInput};
use super::indexer::{DiscoveryWork, SyncProgress};
use super::root_store::{self, LibraryRoot};
use super::scanner;
use crate::engine::audio::formats::stamp_from_metadata;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

const DB_BATCH: usize = 500;

pub fn run(
    conn: &mut Connection,
    root_id: i64,
    progress: &mut impl FnMut(SyncProgress),
) -> Result<DiscoveryWork, String> {
    let root = root_store::get(conn, root_id)?.ok_or("library_root_not_found")?;
    validate(&root)?;
    let root_path = PathBuf::from(&root.path);
    let generation = catalog_store::begin_scan(conn, root_id)?;
    progress(SyncProgress {
        root_id,
        phase: "discovering",
        processed: 0,
        total: 0,
    });
    let known = catalog_store::known_files(conn)?;
    let mut batch = Vec::with_capacity(DB_BATCH);
    let mut pending = Vec::new();
    let mut metadata_errors = 0;
    let mut discovered = 0;
    let mut save_error = None;
    let walk = scanner::discover_each(&root_path, &exclusion_paths(conn, &root), |file| {
        if save_error.is_some() {
            return;
        }
        let previous = known.get(&file.path_key);
        let (mtime, size, changed) = existing_stamp(&file.path, previous, &mut metadata_errors);
        let needs_metadata = changed
            || previous
                .and_then(|old| old.metadata_state.as_deref())
                .map(|state| state == "pending")
                .unwrap_or(true);
        let (relative_path, file_name, extension) = path_fields(&root_path, &file.path);
        if needs_metadata {
            pending.push((file.path_key.clone(), file.path));
        }
        batch.push(CatalogInput {
            path_key: file.path_key,
            relative_path,
            file_name,
            extension,
            mtime,
            size,
            content_changed: changed,
            needs_metadata,
        });
        if batch.len() < DB_BATCH {
            return;
        }
        if let Err(error) =
            catalog_store::save_discovery(conn, root_id, &root.collection, generation, &batch)
        {
            save_error = Some(error);
            return;
        }
        discovered += batch.len();
        batch.clear();
        progress(SyncProgress {
            root_id,
            phase: "cataloging",
            processed: discovered,
            total: 0,
        });
        std::thread::yield_now();
    });
    if let Some(error) = save_error {
        return Err(error);
    }
    if !batch.is_empty() {
        catalog_store::save_discovery(conn, root_id, &root.collection, generation, &batch)?;
        discovered += batch.len();
    }
    progress(SyncProgress {
        root_id,
        phase: "cataloging",
        processed: discovered,
        total: discovered,
    });
    let missing = catalog_store::finish_discovery(conn, root_id, generation)?;
    Ok(DiscoveryWork {
        root_id,
        discovered,
        unchanged: discovered - pending.len(),
        pending,
        missing,
        inaccessible: walk.inaccessible + metadata_errors,
    })
}

fn existing_stamp(
    path: &str,
    previous: Option<&catalog_store::KnownFile>,
    errors: &mut usize,
) -> (i64, i64, bool) {
    let Some(previous) = previous else {
        return (0, 0, true);
    };
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let (mtime, size) = stamp_from_metadata(&metadata);
            (
                mtime,
                size,
                previous.mtime != mtime || previous.size != size,
            )
        }
        Err(_) => {
            *errors += 1;
            (previous.mtime, previous.size, false)
        }
    }
}

fn validate(root: &LibraryRoot) -> Result<(), String> {
    if !root.enabled {
        return Err("library_root_not_enabled".into());
    }
    if !Path::new(&root.path).is_dir() {
        return Err("library_root_unavailable".into());
    }
    Ok(())
}

fn exclusion_paths(conn: &Connection, current: &LibraryRoot) -> Vec<PathBuf> {
    root_store::list(conn)
        .unwrap_or_default()
        .into_iter()
        .filter(|root| {
            root.enabled
                && root.id != current.id
                && Path::new(&root.path_key).starts_with(Path::new(&current.path_key))
        })
        .map(|root| PathBuf::from(root.path))
        .collect()
}

pub fn path_fields(root: &Path, full_path: &str) -> (String, String, String) {
    let path = Path::new(full_path);
    let relative = path.strip_prefix(root).unwrap_or(path);
    let relative_path = relative.to_string_lossy().to_string();
    let file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| relative_path.clone());
    let extension = path
        .extension()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    (relative_path, file_name, extension)
}
