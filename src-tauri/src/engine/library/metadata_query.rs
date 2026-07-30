use crate::domain::library::metadata_fields::LibraryMetadataItem;
use crate::engine::persist::db;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::PathBuf;

const ITEM_SQL: &str = "
SELECT lr.path,lt.relative_path,lt.collection,lt.file_name,
 NULLIF(COALESCE(um.title,lt.title),''),
 NULLIF(COALESCE(um.artist,lt.artist),''),
 NULLIF(COALESCE(um.album,lt.album),''),
 NULLIF(um.album_artist,''),
 NULLIF(COALESCE(um.genre,lt.genre),''),
 CASE WHEN um.year IS NULL THEN lt.year
      WHEN trim(um.year)='' THEN NULL ELSE CAST(um.year AS INTEGER) END,
 CASE WHEN um.track_number IS NULL THEN lt.track_number
      WHEN trim(um.track_number)='' THEN NULL ELSE CAST(um.track_number AS INTEGER) END,
 NULLIF(um.composer,''),NULLIF(um.comment,''),NULLIF(um.display_name,''),
 NULLIF(um.category,''),NULLIF(um.description,''),
 COALESCE((SELECT group_concat(keyword,char(31)) FROM
   (SELECT keyword FROM track_keyword WHERE path_key=lt.path_key
    ORDER BY position,keyword_key)),'')
FROM library_track lt
JOIN library_root lr ON lr.id=lt.root_id
LEFT JOIN track_user_metadata um ON um.path_key=lt.path_key
WHERE lt.path_key=?1 AND lt.present=1 AND lr.enabled=1";

pub fn items(
    connection: &Connection,
    paths: &[String],
) -> Result<Vec<LibraryMetadataItem>, String> {
    let mut statement = connection
        .prepare(ITEM_SQL)
        .map_err(|error| error.to_string())?;
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        let key = db::normalize_key(path);
        let item = statement
            .query_row(params![key], map_item)
            .optional()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "library_track_not_found".to_string())?;
        result.push(item);
    }
    Ok(result)
}

pub fn suggestions(
    connection: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<String>, String> {
    let query = super::search_text::normalize(query);
    let pattern = format!("%{query}%");
    let mut statement = connection
        .prepare(
            "SELECT tk.keyword,COUNT(*) AS uses
             FROM track_keyword tk
             JOIN library_track lt ON lt.path_key=tk.path_key
             JOIN library_root lr ON lr.id=lt.root_id
             WHERE lt.present=1 AND lr.enabled=1
               AND (?1='' OR tk.keyword_key LIKE ?2)
             GROUP BY tk.keyword_key
             ORDER BY uses DESC,tk.keyword COLLATE NOCASE
             LIMIT ?3",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![query, pattern, limit.clamp(1, 100) as i64], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn map_item(row: &Row<'_>) -> rusqlite::Result<LibraryMetadataItem> {
    let root: String = row.get(0)?;
    let relative: String = row.get(1)?;
    let tags: String = row.get(16)?;
    Ok(LibraryMetadataItem {
        path: PathBuf::from(root)
            .join(relative)
            .to_string_lossy()
            .to_string(),
        collection: row.get(2)?,
        file_name: row.get(3)?,
        title: row.get(4)?,
        artist: row.get(5)?,
        album: row.get(6)?,
        album_artist: row.get(7)?,
        genre: row.get(8)?,
        year: row.get(9)?,
        track_number: row.get(10)?,
        composer: row.get(11)?,
        comment: row.get(12)?,
        display_name: row.get(13)?,
        category: row.get(14)?,
        description: row.get(15)?,
        tags: split_tags(&tags),
    })
}

fn split_tags(value: &str) -> Vec<String> {
    value
        .split(char::from(31))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}
