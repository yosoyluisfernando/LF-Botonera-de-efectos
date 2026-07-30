use super::search_result::{fields as candidate_fields, into_result};
pub use super::search_result::{Candidate, SearchResult};
use super::{search_expression, search_score, search_text, tag_roles};
use rusqlite::{params, Connection};

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
    let mut candidates = if query.is_empty() {
        browse_candidates(conn, collection, safe_limit)?
    } else if query.chars().count() <= 2 {
        prefix_candidates(conn, &query, collection, 5_000)?
    } else {
        fts_candidates(conn, &query, collection, 5_000)?
    };
    tag_roles::correct(&mut candidates);
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
         AND lt.present=1 AND lr.enabled=1 AND (?2='' OR lt.collection=?2)
         AND ?4=''
         ORDER BY bm25(library_track_search) LIMIT ?3",
        &expression,
        collection,
        limit,
        "",
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
    let tag_prefix = format!("{query}%");
    query_candidates(
        conn,
        "WHERE (s.search_text LIKE ?1 OR EXISTS(
           SELECT 1 FROM track_keyword tk
           WHERE tk.path_key=lt.path_key AND tk.keyword_key LIKE ?4))
         AND lt.present=1 AND lr.enabled=1
         AND (?2='' OR lt.collection=?2) LIMIT ?3",
        &first,
        collection,
        limit,
        &tag_prefix,
    )
}

fn browse_candidates(
    conn: &Connection,
    collection: &str,
    limit: usize,
) -> Result<Vec<Candidate>, String> {
    query_candidates(
        conn,
        "WHERE lt.present=1 AND lr.enabled=1 AND (?2='' OR lt.collection=?2)
         AND ?4=''
         ORDER BY lt.file_name COLLATE NOCASE LIMIT ?3",
        "",
        collection,
        limit,
        "",
    )
}

fn query_candidates(
    conn: &Connection,
    clause: &str,
    value: &str,
    collection: &str,
    limit: usize,
    tag_prefix: &str,
) -> Result<Vec<Candidate>, String> {
    let sql = format!(
        "SELECT {} FROM library_track_search s
         JOIN library_track lt ON lt.path_key=s.path_key
         JOIN library_root lr ON lr.id=lt.root_id
         JOIN track t ON t.path=lt.path_key
         LEFT JOIN track_user_metadata um ON um.path_key=lt.path_key {clause}",
        candidate_fields("s.search_text")
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![value, collection, limit as i64, tag_prefix],
            super::search_result::map,
        )
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
