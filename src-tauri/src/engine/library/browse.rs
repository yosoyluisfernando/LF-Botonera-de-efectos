use super::search::{into_result, map_candidate, Candidate, SearchResult};
use super::service::LibraryService;
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
        limit: usize,
        cursor: Option<&BrowseCursor>,
        direction: &str,
    ) -> Result<BrowsePage, String> {
        browse(
            &self.connection()?,
            collection,
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
    let collection = collection.unwrap_or("");
    if !collection.is_empty() && !matches!(collection, "music" | "effects") {
        return Err("invalid_library_collection".into());
    }
    let safe_limit = limit.clamp(1, 500);
    let cursor_name = cursor.map(|value| value.file_name.as_str()).unwrap_or("");
    let cursor_key = cursor.map(|value| value.path_key.as_str()).unwrap_or("");
    let collection_clause = if collection.is_empty() {
        "lt.present=1"
    } else {
        "lt.collection=?1 AND lt.present=1"
    };
    let (comparison, order) = match direction {
        BrowseDirection::Forward => (">", "ASC"),
        BrowseDirection::Backward => ("<", "DESC"),
    };
    let cursor_clause = if cursor.is_some() {
        format!(
            "(lt.file_name COLLATE NOCASE {comparison} ?2 COLLATE NOCASE
              OR (lt.file_name COLLATE NOCASE = ?2 COLLATE NOCASE
                  AND lt.path_key{comparison}?3))"
        )
    } else {
        "1=1".to_string()
    };
    let sql = format!(
        "SELECT lr.path,lt.relative_path,lt.collection,lt.file_name,lt.title,
         lt.artist,lt.album,lt.genre,lt.year,lt.track_number,t.duration_s,
         lt.metadata_state,'' AS search_text,lt.path_key
         FROM library_track lt
         JOIN library_root lr ON lr.id=lt.root_id
         JOIN track t ON t.path=lt.path_key
         WHERE {collection_clause}
         AND {cursor_clause}
         ORDER BY lt.file_name COLLATE NOCASE {order},lt.path_key {order}
         LIMIT ?4"
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![collection, cursor_name, cursor_key, safe_limit as i64 + 1],
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
    let previous_cursor = candidates.first().map(candidate_cursor);
    let next_cursor = candidates.last().map(candidate_cursor);
    Ok(BrowsePage {
        items: candidates.into_iter().map(into_result).collect(),
        previous_cursor,
        next_cursor,
        has_more,
    })
}

fn candidate_cursor(candidate: &Candidate) -> BrowseCursor {
    BrowseCursor {
        file_name: candidate.result.file_name.clone(),
        path_key: candidate.path_key.clone(),
    }
}

#[cfg(test)]
#[path = "browse_tests.rs"]
mod tests;
