//! Traslada el catálogo de subraíces unificadas a su nueva raíz.
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::library::search_text;
use rusqlite::{params, Transaction};
use std::path::Path;

pub fn merge_children(
    transaction: &Transaction<'_>,
    root_id: i64,
    display_path: &str,
    collection: LibraryCollection,
    merged_ids: &[i64],
) -> Result<(), String> {
    for merged in merged_ids {
        let child_path = transaction
            .query_row(
                "SELECT path FROM library_root WHERE id=?1",
                params![merged],
                |row| row.get::<_, String>(0),
            )
            .map_err(|error| error.to_string())?;
        let relative = Path::new(&child_path)
            .strip_prefix(Path::new(display_path))
            .map_err(|_| "library_root_merge_path")?;
        let prefix = format!(
            "{}{}",
            relative.to_string_lossy(),
            std::path::MAIN_SEPARATOR
        );
        let search_prefix = search_text::normalize(&relative.to_string_lossy());
        transaction
            .execute(
                "UPDATE library_track_search SET search_text=?1 || ' ' || search_text
                 WHERE path_key IN (SELECT path_key FROM library_track WHERE root_id=?2)",
                params![search_prefix, merged],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "UPDATE library_track SET root_id=?1,collection=?2,
                 relative_path=?4 || relative_path WHERE root_id=?3",
                params![root_id, collection.as_str(), merged, prefix],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute("DELETE FROM library_root WHERE id=?1", params![merged])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
