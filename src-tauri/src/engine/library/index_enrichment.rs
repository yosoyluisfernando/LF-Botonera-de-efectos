//! Segunda fase: completa duración y etiquetas cuando los nombres ya son buscables.
use super::catalog_store::{self, MetadataUpdate};
use super::indexer::{DiscoveryWork, SyncProgress};
use super::metadata_batch;
use crate::engine::audio::formats::file_stamp;
use rusqlite::Connection;
use std::path::Path;

const DB_BATCH: usize = 500;

pub fn run(
    conn: &mut Connection,
    work: &DiscoveryWork,
    offset: usize,
    total: usize,
    progress: &mut impl FnMut(SyncProgress),
) -> Result<usize, String> {
    let mut failed = 0;
    for (index, chunk) in work.pending.chunks(DB_BATCH).enumerate() {
        let paths = chunk
            .iter()
            .map(|(_, path)| path.clone())
            .collect::<Vec<_>>();
        let results = metadata_batch::read(&paths, metadata_batch::recommended_workers());
        let updates = chunk
            .iter()
            .zip(results)
            .map(|((key, path), result)| {
                failed += usize::from(result.is_err());
                let (mtime, size) = file_stamp(Path::new(path));
                MetadataUpdate {
                    path_key: key.clone(),
                    mtime,
                    size,
                    result,
                }
            })
            .collect::<Vec<_>>();
        catalog_store::save_metadata_batch(conn, &updates)?;
        progress(SyncProgress {
            root_id: work.root_id,
            phase: "enriching",
            processed: offset + ((index + 1) * DB_BATCH).min(work.pending.len()),
            total,
        });
        std::thread::yield_now();
    }
    catalog_store::finish_enrichment(conn, work.root_id)?;
    Ok(failed)
}
