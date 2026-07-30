//! Reconstrucción única del texto FTS efectivo de una pista.
use rusqlite::{params, Transaction};

pub fn rebuild(transaction: &Transaction<'_>, path_key: &str) -> Result<(), String> {
    let text: String = transaction
        .query_row(
            "SELECT lt.file_name||' '||lt.relative_path||' '||
             COALESCE(um.title,lt.title,'')||' '||
             COALESCE(um.artist,lt.artist,'')||' '||
             COALESCE(um.album,lt.album,'')||' '||
             COALESCE(um.album_artist,'')||' '||
             COALESCE(um.genre,lt.genre,'')||' '||
             COALESCE(um.year,CAST(lt.year AS TEXT),'')||' '||
             COALESCE(um.track_number,CAST(lt.track_number AS TEXT),'')||' '||
             COALESCE(um.composer,'')||' '||COALESCE(um.comment,'')||' '||
             COALESCE(um.display_name,'')||' '||COALESCE(um.category,'')||' '||
             COALESCE(um.description,'')||' '||
             COALESCE((SELECT group_concat(keyword,' ') FROM track_keyword
                       WHERE path_key=lt.path_key),'')
             FROM library_track lt
             LEFT JOIN track_user_metadata um ON um.path_key=lt.path_key
             WHERE lt.path_key=?1",
            params![path_key],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    replace(transaction, path_key, &super::search_text::normalize(&text))
}

pub fn replace(transaction: &Transaction<'_>, path_key: &str, text: &str) -> Result<(), String> {
    transaction
        .execute(
            "DELETE FROM library_track_search WHERE path_key=?1",
            params![path_key],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "INSERT INTO library_track_search(path_key,search_text) VALUES(?1,?2)",
            params![path_key, text],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}
