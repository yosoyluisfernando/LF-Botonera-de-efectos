use super::{CatalogInput, MetadataUpdate};
use crate::engine::library::{search_text, time::now_epoch};
use rusqlite::{params, Transaction};

pub(super) fn upsert_track_placeholder(
    tx: &Transaction,
    file: &CatalogInput,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
         VALUES(?1,?2,?3,-1,0,0)
         ON CONFLICT(path) DO UPDATE SET mtime=excluded.mtime,size=excluded.size,
         duration_s=-1,sample_rate=0,channels=0,norm_gain_db=0,
         measured_peak_db=NULL,measured_lufs=NULL,analyzed_at=NULL",
        params![file.path_key, file.mtime, file.size],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub(super) fn upsert_catalog(
    tx: &Transaction,
    root_id: i64,
    collection: &str,
    generation: i64,
    file: &CatalogInput,
) -> Result<(), String> {
    let clear = file.content_changed as i64;
    tx.execute(
        "INSERT INTO library_track(path_key,root_id,collection,relative_path,file_name,
         extension,metadata_state,scan_generation,last_seen_at)
         VALUES(?1,?2,?3,?4,?5,?6,'pending',?7,?8)
         ON CONFLICT(path_key) DO UPDATE SET root_id=excluded.root_id,
         collection=excluded.collection,relative_path=excluded.relative_path,
         file_name=excluded.file_name,extension=excluded.extension,present=1,
         scan_generation=excluded.scan_generation,last_seen_at=excluded.last_seen_at,
         title=CASE ?9 WHEN 1 THEN NULL ELSE title END,
         artist=CASE ?9 WHEN 1 THEN NULL ELSE artist END,
         album=CASE ?9 WHEN 1 THEN NULL ELSE album END,
         genre=CASE ?9 WHEN 1 THEN NULL ELSE genre END,
         year=CASE ?9 WHEN 1 THEN NULL ELSE year END,
         track_number=CASE ?9 WHEN 1 THEN NULL ELSE track_number END,
         tags_readable=CASE ?9 WHEN 1 THEN 0 ELSE tags_readable END,
         metadata_state=CASE ?9 WHEN 1 THEN 'pending' ELSE metadata_state END",
        params![
            file.path_key,
            root_id,
            collection,
            file.relative_path,
            file.file_name,
            file.extension,
            generation,
            now_epoch(),
            clear
        ],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub(super) fn rebuild_search(tx: &Transaction, path_key: &str) -> Result<(), String> {
    let text: String = tx
        .query_row(
            "SELECT file_name||' '||relative_path||' '||COALESCE(title,'')||' '||
             COALESCE(artist,'')||' '||COALESCE(album,'')||' '||COALESCE(genre,'')
             FROM library_track WHERE path_key=?1",
            params![path_key],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    replace_search(tx, path_key, &search_text::normalize(&text))
}

pub(super) fn apply_metadata(tx: &Transaction, update: &MetadataUpdate) -> Result<(), String> {
    let path_key = &update.path_key;
    match &update.result {
        Ok(metadata) => {
            tx.execute(
                "UPDATE track SET duration_s=?2,sample_rate=?3,channels=?4 WHERE path=?1",
                params![
                    path_key,
                    metadata.duration_s,
                    metadata.sample_rate,
                    metadata.channels
                ],
            )
            .map_err(|error| error.to_string())?;
            tx.execute(
                "UPDATE library_track SET title=?2,artist=?3,album=?4,genre=?5,
                 year=?6,track_number=?7,tags_readable=?8,metadata_state=?9
                 WHERE path_key=?1",
                params![
                    path_key,
                    metadata.title,
                    metadata.artist,
                    metadata.album,
                    metadata.genre,
                    metadata.year,
                    metadata.track,
                    metadata.tags_readable as i64,
                    if metadata.tags_readable {
                        "complete"
                    } else {
                        "partial"
                    }
                ],
            )
            .map_err(|error| error.to_string())?;
            rebuild_search(tx, path_key)
        }
        Err(_) => tx
            .execute(
                "UPDATE library_track SET metadata_state='failed' WHERE path_key=?1",
                params![path_key],
            )
            .map(|_| ())
            .map_err(|error| error.to_string()),
    }
}

pub(super) fn replace_search(tx: &Transaction, path_key: &str, text: &str) -> Result<(), String> {
    tx.execute(
        "DELETE FROM library_track_search WHERE path_key=?1",
        params![path_key],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO library_track_search(path_key,search_text) VALUES(?1,?2)",
        params![path_key, text],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}
