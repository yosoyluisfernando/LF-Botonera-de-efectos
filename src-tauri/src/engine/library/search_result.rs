use rusqlite::{Result as SqlResult, Row};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub path_key: String,
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

pub struct Candidate {
    pub root_path: String,
    pub relative_path: String,
    pub result: SearchResult,
    pub search_text: String,
    pub user_roles: bool,
}

pub fn fields(search_text: &str) -> String {
    format!(
        "lr.path,lt.relative_path,lt.collection,lt.file_name,
         NULLIF(COALESCE(um.title,lt.title),''),
         NULLIF(COALESCE(um.artist,lt.artist),''),
         NULLIF(COALESCE(um.album,lt.album),''),
         NULLIF(COALESCE(um.genre,lt.genre),''),
         CASE WHEN um.year IS NULL THEN lt.year
              WHEN trim(um.year)='' THEN NULL ELSE CAST(um.year AS INTEGER) END,
         CASE WHEN um.track_number IS NULL THEN lt.track_number
              WHEN trim(um.track_number)='' THEN NULL
              ELSE CAST(um.track_number AS INTEGER) END,
         t.duration_s,lt.metadata_state,{search_text},lt.path_key,
         (um.title IS NOT NULL OR um.artist IS NOT NULL)"
    )
}

pub fn map(row: &Row<'_>) -> SqlResult<Candidate> {
    Ok(Candidate {
        root_path: row.get(0)?,
        relative_path: row.get(1)?,
        result: SearchResult {
            path: String::new(),
            path_key: row.get(13)?,
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
        user_roles: row.get(14)?,
    })
}

pub fn into_result(mut candidate: Candidate) -> SearchResult {
    candidate.result.path = PathBuf::from(candidate.root_path)
        .join(candidate.relative_path)
        .to_string_lossy()
        .to_string();
    candidate.result
}
