//! Orquesta descubrimiento rápido primero y metadatos después.
use super::{catalog_store, index_discovery, index_enrichment};
use rusqlite::Connection;
use serde::Serialize;

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

pub(super) struct DiscoveryWork {
    pub root_id: i64,
    pub discovered: usize,
    pub unchanged: usize,
    pub pending: Vec<(String, String)>,
    pub missing: usize,
    pub inaccessible: usize,
}

pub fn sync_root<F>(conn: &mut Connection, root_id: i64, progress: F) -> Result<SyncReport, String>
where
    F: FnMut(SyncProgress),
{
    sync_roots(conn, &[root_id], progress)?
        .into_iter()
        .next()
        .ok_or_else(|| "library_sync_empty".into())
}

pub fn sync_roots<F>(
    conn: &mut Connection,
    root_ids: &[i64],
    mut progress: F,
) -> Result<Vec<SyncReport>, String>
where
    F: FnMut(SyncProgress),
{
    let mut work = Vec::with_capacity(root_ids.len());
    for root_id in root_ids {
        match index_discovery::run(conn, *root_id, &mut progress) {
            Ok(discovery) => work.push(discovery),
            Err(error) => {
                catalog_store::mark_root_error(conn, *root_id);
                return Err(error);
            }
        }
    }
    let searchable = work.iter().map(|item| item.discovered).sum();
    progress(SyncProgress {
        root_id: 0,
        phase: "catalog_ready",
        processed: searchable,
        total: searchable,
    });
    enrich_all(conn, &work, &mut progress)
}

fn enrich_all(
    conn: &mut Connection,
    work: &[DiscoveryWork],
    progress: &mut impl FnMut(SyncProgress),
) -> Result<Vec<SyncReport>, String> {
    let total = work.iter().map(|item| item.pending.len()).sum();
    let mut offset = 0;
    let mut reports = Vec::with_capacity(work.len());
    for item in work {
        let failed = match index_enrichment::run(conn, item, offset, total, progress) {
            Ok(failed) => failed,
            Err(error) => {
                catalog_store::mark_root_error(conn, item.root_id);
                return Err(error);
            }
        };
        reports.push(SyncReport {
            root_id: item.root_id,
            discovered: item.discovered,
            enriched: item.pending.len() - failed,
            unchanged: item.unchanged,
            failed,
            missing: item.missing,
            inaccessible: item.inaccessible,
        });
        offset += item.pending.len();
    }
    Ok(reports)
}

pub(super) use index_discovery::path_fields;

#[cfg(test)]
#[path = "indexer_tests.rs"]
mod tests;
