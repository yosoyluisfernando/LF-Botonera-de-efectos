use super::catalog_store::{self, CatalogInput, MetadataUpdate};
use super::{indexer, metadata, root_store};
use crate::domain::library::{owner_for_path, LibraryCollection, RootSpec};
use crate::engine::audio::formats::{is_audio_path, stamp_from_metadata};
use rusqlite::Connection;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct UpdateReport {
    pub updated: usize,
    pub removed: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub reconcile_roots: Vec<i64>,
}

pub fn apply_paths(conn: &mut Connection, paths: &[PathBuf]) -> Result<UpdateReport, String> {
    let roots = root_store::list(conn)?
        .into_iter()
        .filter(|root| root.enabled)
        .collect::<Vec<_>>();
    let specs = roots
        .iter()
        .map(|root| {
            Ok(RootSpec {
                id: root.id,
                path_key: root.path_key.clone(),
                collection: LibraryCollection::parse(&root.collection)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let by_id = roots
        .iter()
        .map(|root| (root.id, root))
        .collect::<HashMap<_, _>>();
    let mut report = UpdateReport::default();
    let mut seen = HashSet::new();
    let mut reconcile = BTreeSet::new();
    for path in paths {
        let Ok((display, path_key)) = super::root_path::normalize_path(path) else {
            continue;
        };
        if !seen.insert(path_key.clone()) {
            continue;
        }
        if path.is_dir() {
            if let Some(owner) = owner_for_path(&specs, &path_key) {
                reconcile.insert(owner.id);
            }
            continue;
        }
        if !path.is_file() || !is_audio_path(path) {
            report.removed += catalog_store::mark_path_missing(conn, &path_key)?;
            continue;
        }
        let Some(owner) = owner_for_path(&specs, &path_key) else {
            continue;
        };
        let root = by_id[&owner.id];
        update_file(conn, root, &display, &path_key, &mut report)?;
    }
    report.reconcile_roots = reconcile.into_iter().collect();
    Ok(report)
}

fn update_file(
    conn: &mut Connection,
    root: &root_store::LibraryRoot,
    display: &str,
    path_key: &str,
    report: &mut UpdateReport,
) -> Result<(), String> {
    let file_metadata = std::fs::metadata(display).map_err(|error| error.to_string())?;
    let (mtime, size) = stamp_from_metadata(&file_metadata);
    let (relative_path, file_name, extension) =
        indexer::path_fields(Path::new(&root.path), display);
    let input = CatalogInput {
        path_key: path_key.to_owned(),
        relative_path,
        file_name,
        extension,
        mtime,
        size,
        // Esta ruta procede de un evento real del SO. Aunque tamaño y `mtime`
        // coincidan por resolución del sistema de archivos, el contenido pudo
        // cambiar; se invalida y relee una sola vez por ráfaga debounced.
        content_changed: true,
        needs_metadata: true,
    };
    catalog_store::save_discovery(
        conn,
        root.id,
        &root.collection,
        root.scan_generation,
        &[input],
    )?;
    let update = MetadataUpdate {
        path_key: path_key.to_owned(),
        mtime,
        size,
        result: metadata::read(display),
    };
    report.failed += usize::from(update.result.is_err());
    catalog_store::save_metadata_batch(conn, &[update])?;
    report.updated += 1;
    Ok(())
}

#[cfg(test)]
#[path = "incremental_tests.rs"]
mod tests;
