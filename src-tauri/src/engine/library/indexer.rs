use super::catalog_store::{self, CatalogInput, MetadataUpdate};
use super::metadata_batch;
use super::root_store::{self, LibraryRoot};
use super::scanner;
use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};

const DB_BATCH: usize = 500;

#[derive(Debug, Clone, Serialize)]
pub struct SyncProgress {
    pub root_id: i64,
    pub phase: &'static str,
    pub processed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub root_id: i64,
    pub discovered: usize,
    pub enriched: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub missing: usize,
    pub inaccessible: usize,
}

pub fn sync_root<F>(
    conn: &mut Connection,
    root_id: i64,
    mut progress: F,
) -> Result<SyncReport, String>
where
    F: FnMut(SyncProgress),
{
    let result = sync_root_inner(conn, root_id, &mut progress);
    if result.is_err() {
        catalog_store::mark_root_error(conn, root_id);
    }
    result
}

fn sync_root_inner(
    conn: &mut Connection,
    root_id: i64,
    progress: &mut impl FnMut(SyncProgress),
) -> Result<SyncReport, String> {
    let root = root_store::get(conn, root_id)?.ok_or("library_root_not_found")?;
    if !root.enabled {
        return Err("library_root_not_enabled".into());
    }
    let root_path = PathBuf::from(&root.path);
    if !root_path.is_dir() {
        return Err("library_root_unavailable".into());
    }
    let exclusions = exclusion_paths(conn, &root);
    let generation = catalog_store::begin_scan(conn, root_id)?;
    progress(SyncProgress {
        root_id,
        phase: "discovering",
        processed: 0,
        total: 0,
    });
    let discovery = scanner::discover(&root_path, &exclusions);
    let known = catalog_store::known_files(conn)?;
    let mut inputs = Vec::with_capacity(discovery.files.len());
    let mut pending = Vec::new();
    for file in discovery.files {
        let previous = known.get(&file.path_key);
        let content_changed = previous
            .map(|old| old.mtime != file.mtime || old.size != file.size)
            .unwrap_or(true);
        let needs_metadata = content_changed
            || previous
                .and_then(|old| old.metadata_state.as_deref())
                .map(|state| state == "pending")
                .unwrap_or(true);
        let (relative_path, file_name, extension) = path_fields(&root_path, &file.path);
        if needs_metadata {
            pending.push((file.path_key.clone(), file.path));
        }
        inputs.push(CatalogInput {
            path_key: file.path_key,
            relative_path,
            file_name,
            extension,
            mtime: file.mtime,
            size: file.size,
            content_changed,
            needs_metadata,
        });
    }
    for (index, chunk) in inputs.chunks(DB_BATCH).enumerate() {
        catalog_store::save_discovery(conn, root_id, &root.collection, generation, chunk)?;
        progress(SyncProgress {
            root_id,
            phase: "cataloging",
            processed: ((index + 1) * DB_BATCH).min(inputs.len()),
            total: inputs.len(),
        });
    }
    let failed = enrich(conn, root_id, &pending, progress)?;
    let missing = catalog_store::finish_scan(conn, root_id, generation)?;
    Ok(SyncReport {
        root_id,
        discovered: inputs.len(),
        enriched: pending.len() - failed,
        unchanged: inputs.len() - pending.len(),
        failed,
        missing,
        inaccessible: discovery.walk.inaccessible + discovery.metadata_errors,
    })
}

fn enrich(
    conn: &mut Connection,
    root_id: i64,
    pending: &[(String, String)],
    progress: &mut impl FnMut(SyncProgress),
) -> Result<usize, String> {
    let mut failed = 0;
    for (index, chunk) in pending.chunks(DB_BATCH).enumerate() {
        let paths = chunk
            .iter()
            .map(|(_, path)| path.clone())
            .collect::<Vec<_>>();
        let results = metadata_batch::read(&paths, metadata_batch::recommended_workers());
        let updates = chunk
            .iter()
            .zip(results)
            .map(|((key, _), result)| {
                failed += usize::from(result.is_err());
                MetadataUpdate {
                    path_key: key.clone(),
                    result,
                }
            })
            .collect::<Vec<_>>();
        catalog_store::save_metadata_batch(conn, &updates)?;
        progress(SyncProgress {
            root_id,
            phase: "enriching",
            processed: ((index + 1) * DB_BATCH).min(pending.len()),
            total: pending.len(),
        });
    }
    Ok(failed)
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

pub(super) fn path_fields(root: &Path, full_path: &str) -> (String, String, String) {
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

#[cfg(test)]
#[path = "indexer_tests.rs"]
mod tests;
