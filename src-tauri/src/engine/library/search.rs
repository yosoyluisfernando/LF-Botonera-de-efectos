use super::{search_expression, search_score, search_text};
use rusqlite::{params, Connection, Row};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub collection: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub duration_s: f64,
    pub metadata_state: String,
}

struct Candidate {
    root_path: String,
    relative_path: String,
    result: SearchResult,
    search_text: String,
}

pub fn search(
    conn: &Connection,
    query: &str,
    collection: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let query = search_text::normalize(query);
    let collection = collection.unwrap_or("");
    if !collection.is_empty() && !matches!(collection, "music" | "effects") {
        return Err("invalid_library_collection".into());
    }
    let safe_limit = limit.clamp(1, 500);
    let candidates = if query.is_empty() {
        browse_candidates(conn, collection, safe_limit)?
    } else if query.chars().count() <= 4 {
        prefix_candidates(conn, &query, collection, 5_000)?
    } else {
        fts_candidates(conn, &query, collection, 5_000)?
    };
    let mut scored = candidates
        .into_iter()
        .filter_map(|candidate| {
            let score = search_score::score(
                &query,
                &candidate.result.file_name,
                candidate.result.title.as_deref(),
                candidate.result.artist.as_deref(),
                candidate.result.album.as_deref(),
                candidate.result.genre.as_deref(),
                &candidate.search_text,
            )?;
            Some((score, candidate))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.result.file_name.cmp(&right.result.file_name))
            .then_with(|| left.result.path.cmp(&right.result.path))
    });
    scored.truncate(safe_limit);
    Ok(scored
        .into_iter()
        .map(|(_, candidate)| into_result(candidate))
        .collect())
}

fn fts_candidates(
    conn: &Connection,
    query: &str,
    collection: &str,
    limit: usize,
) -> Result<Vec<Candidate>, String> {
    let expression = search_expression::trigram(query);
    if expression.is_empty() {
        return prefix_candidates(conn, query, collection, limit);
    }
    query_candidates(
        conn,
        "WHERE library_track_search MATCH ?1
         AND lt.present=1 AND (?2='' OR lt.collection=?2)
         ORDER BY bm25(library_track_search) LIMIT ?3",
        &expression,
        collection,
        limit,
    )
}

fn prefix_candidates(
    conn: &Connection,
    query: &str,
    collection: &str,
    limit: usize,
) -> Result<Vec<Candidate>, String> {
    let first = query
        .chars()
        .next()
        .map(|value| format!("{value}%"))
        .unwrap_or_default();
    query_candidates(
        conn,
        "WHERE s.search_text LIKE ?1 AND lt.present=1
         AND (?2='' OR lt.collection=?2) LIMIT ?3",
        &first,
        collection,
        limit,
    )
}

fn browse_candidates(
    conn: &Connection,
    collection: &str,
    limit: usize,
) -> Result<Vec<Candidate>, String> {
    query_candidates(
        conn,
        "WHERE lt.present=1 AND (?2='' OR lt.collection=?2)
         ORDER BY lt.file_name COLLATE NOCASE LIMIT ?3",
        "",
        collection,
        limit,
    )
}

fn query_candidates(
    conn: &Connection,
    clause: &str,
    value: &str,
    collection: &str,
    limit: usize,
) -> Result<Vec<Candidate>, String> {
    let sql = format!(
        "SELECT lr.path,lt.relative_path,lt.collection,lt.file_name,lt.title,lt.artist,
         lt.album,lt.genre,lt.year,lt.track_number,t.duration_s,lt.metadata_state,
         s.search_text FROM library_track_search s
         JOIN library_track lt ON lt.path_key=s.path_key
         JOIN library_root lr ON lr.id=lt.root_id
         JOIN track t ON t.path=lt.path_key {clause}"
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![value, collection, limit as i64], map_candidate)
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn map_candidate(row: &Row) -> rusqlite::Result<Candidate> {
    Ok(Candidate {
        root_path: row.get(0)?,
        relative_path: row.get(1)?,
        result: SearchResult {
            path: String::new(),
            collection: row.get(2)?,
            file_name: row.get(3)?,
            title: row.get(4)?,
            artist: row.get(5)?,
            album: row.get(6)?,
            genre: row.get(7)?,
            year: row.get(8)?,
            track_number: row.get(9)?,
            duration_s: row.get(10)?,
            metadata_state: row.get(11)?,
        },
        search_text: row.get(12)?,
    })
}

fn into_result(mut candidate: Candidate) -> SearchResult {
    candidate.result.path = PathBuf::from(candidate.root_path)
        .join(candidate.relative_path)
        .to_string_lossy()
        .to_string();
    candidate.result
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
