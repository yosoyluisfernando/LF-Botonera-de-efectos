use super::search::{Candidate, SearchResult};
use super::search_result::{fields as candidate_fields, into_result, map as map_candidate};
use super::service::LibraryService;
use super::tag_roles;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowseCursor {
    pub file_name: String,
    pub path_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowsePage {
    pub items: Vec<SearchResult>,
    pub previous_cursor: Option<BrowseCursor>,
    pub next_cursor: Option<BrowseCursor>,
    pub has_more: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrowseDirection {
    Forward,
    Backward,
}

impl BrowseDirection {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "forward" => Ok(Self::Forward),
            "backward" => Ok(Self::Backward),
            _ => Err("invalid_library_browse_direction".into()),
        }
    }
}

impl LibraryService {
    pub fn browse(
        &self,
        collection: Option<&str>,
        root_id: Option<i64>,
        relative_prefix: Option<&str>,
        limit: usize,
        cursor: Option<&BrowseCursor>,
        direction: &str,
    ) -> Result<BrowsePage, String> {
        browse_scoped(
            &self.connection()?,
            collection,
            root_id,
            relative_prefix,
            limit,
            cursor,
            BrowseDirection::parse(direction)?,
        )
    }
}

pub fn browse(
    connection: &Connection,
    collection: Option<&str>,
    limit: usize,
    cursor: Option<&BrowseCursor>,
    direction: BrowseDirection,
) -> Result<BrowsePage, String> {
    browse_scoped(connection, collection, None, None, limit, cursor, direction)
}

pub fn browse_scoped(
    connection: &Connection,
    collection: Option<&str>,
    root_id: Option<i64>,
    relative_prefix: Option<&str>,
    limit: usize,
    cursor: Option<&BrowseCursor>,
    direction: BrowseDirection,
) -> Result<BrowsePage, String> {
    let collection = collection.unwrap_or("");
    if !collection.is_empty() && !matches!(collection, "music" | "effects") {
        return Err("invalid_library_collection".into());
    }
    let safe_limit = limit.clamp(1, 500);
    let root_id = root_id.unwrap_or(-1);
    let relative_pattern = relative_pattern(relative_prefix.unwrap_or(""));
    let cursor_name = cursor.map(|value| value.file_name.as_str()).unwrap_or("");
    let cursor_key = cursor.map(|value| value.path_key.as_str()).unwrap_or("");
    let (comparison, order) = match direction {
        BrowseDirection::Forward => (">", "ASC"),
        BrowseDirection::Backward => ("<", "DESC"),
    };
    let cursor_clause = if cursor.is_some() {
        format!(
            "(lt.file_name COLLATE NOCASE {comparison} ?4 COLLATE NOCASE
              OR (lt.file_name COLLATE NOCASE = ?4 COLLATE NOCASE
                  AND lt.path_key{comparison}?5))"
        )
    } else {
        "1=1".to_string()
    };
    let sql = format!(
        "SELECT {}
         FROM library_track lt
         JOIN library_root lr ON lr.id=lt.root_id
         JOIN track t ON t.path=lt.path_key
         LEFT JOIN track_user_metadata um ON um.path_key=lt.path_key
         WHERE lt.present=1 AND lr.enabled=1 AND (?1='' OR lt.collection=?1)
         AND (?2<0 OR lt.root_id=?2)
         AND (?3='' OR replace(lt.relative_path,'\\','/') LIKE ?3 ESCAPE '\\')
         AND {cursor_clause}
         ORDER BY lt.file_name COLLATE NOCASE {order},lt.path_key {order}
         LIMIT ?6",
        candidate_fields("''")
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![
                collection,
                root_id,
                relative_pattern,
                cursor_name,
                cursor_key,
                safe_limit as i64 + 1
            ],
            map_candidate,
        )
        .map_err(|error| error.to_string())?;
    let mut candidates = rows
        .collect::<Result<Vec<Candidate>, _>>()
        .map_err(|error| error.to_string())?;
    let has_more = candidates.len() > safe_limit;
    candidates.truncate(safe_limit);
    if direction == BrowseDirection::Backward {
        candidates.reverse();
    }
    tag_roles::correct(&mut candidates);
    let previous_cursor = candidates.first().map(candidate_cursor);
    let next_cursor = candidates.last().map(candidate_cursor);
    Ok(BrowsePage {
        items: candidates.into_iter().map(into_result).collect(),
        previous_cursor,
        next_cursor,
        has_more,
    })
}

pub(super) fn relative_pattern(prefix: &str) -> String {
    let prefix = prefix.replace('\\', "/").trim_matches('/').to_string();
    if prefix.is_empty() {
        return String::new();
    }
    format!(
        "{}/%",
        prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn candidate_cursor(candidate: &Candidate) -> BrowseCursor {
    BrowseCursor {
        file_name: candidate.result.file_name.clone(),
        path_key: candidate.result.path_key.clone(),
    }
}

#[cfg(test)]
#[path = "browse_tests.rs"]
mod tests;
