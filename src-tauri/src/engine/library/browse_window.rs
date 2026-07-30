//! Ventana por posición absoluta para una barra de desplazamiento de tamaño estable.
use super::browse::relative_pattern;
use super::catalog_count;
use super::search::{Candidate, SearchResult};
use super::search_result::{fields as candidate_fields, into_result, map as map_candidate};
use super::service::LibraryService;
use super::tag_roles;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BrowseWindow {
    pub items: Vec<SearchResult>,
    pub total: usize,
    pub offset: usize,
}

impl LibraryService {
    pub fn browse_window(
        &self,
        collection: Option<&str>,
        root_id: Option<i64>,
        relative_prefix: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Result<BrowseWindow, String> {
        browse_window(
            &self.connection()?,
            collection,
            root_id,
            relative_prefix,
            offset,
            limit,
        )
    }
}

pub fn browse_window(
    connection: &Connection,
    collection: Option<&str>,
    root_id: Option<i64>,
    relative_prefix: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<BrowseWindow, String> {
    let collection = collection.unwrap_or("");
    let total = catalog_count::count(connection, Some(collection), root_id, relative_prefix)?;
    let safe_limit = limit.clamp(1, 500);
    let safe_offset = offset.min(total.saturating_sub(safe_limit));
    let sql = format!(
        "SELECT {}
             FROM library_track lt
             JOIN library_root lr ON lr.id=lt.root_id
             JOIN track t ON t.path=lt.path_key
             LEFT JOIN track_user_metadata um ON um.path_key=lt.path_key
             WHERE lt.present=1 AND lr.enabled=1 AND (?1='' OR lt.collection=?1)
             AND (?2<0 OR lt.root_id=?2)
             AND (?3='' OR replace(lt.relative_path,'\\','/') LIKE ?3 ESCAPE '\\')
             ORDER BY lt.file_name COLLATE NOCASE,lt.path_key
             LIMIT ?4 OFFSET ?5",
        candidate_fields("''")
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![
                collection,
                root_id.unwrap_or(-1),
                relative_pattern(relative_prefix.unwrap_or("")),
                safe_limit as i64,
                safe_offset as i64
            ],
            map_candidate,
        )
        .map_err(|error| error.to_string())?;
    let mut candidates = rows
        .collect::<Result<Vec<Candidate>, _>>()
        .map_err(|error| error.to_string())?;
    tag_roles::correct(&mut candidates);
    Ok(BrowseWindow {
        items: candidates.into_iter().map(into_result).collect(),
        total,
        offset: safe_offset,
    })
}

#[cfg(test)]
#[path = "browse_window_tests.rs"]
mod tests;
