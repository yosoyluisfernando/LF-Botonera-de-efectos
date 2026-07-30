//! Conteo total del ámbito visible, independiente del bloque de carga perezosa.
use super::browse::relative_pattern;
use super::service::LibraryService;
use rusqlite::{params, Connection};

impl LibraryService {
    pub fn count(
        &self,
        collection: Option<&str>,
        root_id: Option<i64>,
        relative_prefix: Option<&str>,
    ) -> Result<usize, String> {
        count(&self.connection()?, collection, root_id, relative_prefix)
    }
}

pub(super) fn count(
    connection: &Connection,
    collection: Option<&str>,
    root_id: Option<i64>,
    relative_prefix: Option<&str>,
) -> Result<usize, String> {
    let collection = collection.unwrap_or("");
    if !collection.is_empty() && !matches!(collection, "music" | "effects") {
        return Err("invalid_library_collection".into());
    }
    connection
        .query_row(
            "SELECT COUNT(*) FROM library_track lt
             JOIN library_root lr ON lr.id=lt.root_id
             WHERE lt.present=1 AND lr.enabled=1
             AND (?1='' OR lt.collection=?1)
             AND (?2<0 OR lt.root_id=?2)
             AND (?3='' OR replace(lt.relative_path,'\\','/') LIKE ?3 ESCAPE '\\')",
            params![
                collection,
                root_id.unwrap_or(-1),
                relative_pattern(relative_prefix.unwrap_or(""))
            ],
            |row| row.get::<_, usize>(0),
        )
        .map_err(|error| error.to_string())
}
