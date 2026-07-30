use crate::domain::library::metadata_fields::{clean_tag, LibraryMetadataFields, FIELD_NAMES};
use crate::engine::persist::db;
use rusqlite::{params, Connection, Transaction};
use std::collections::HashSet;

pub fn save(
    connection: &mut Connection,
    paths: &[String],
    fields: &LibraryMetadataFields,
    add_tags: &[String],
    remove_tags: &[String],
) -> Result<(), String> {
    validate_paths(paths)?;
    let values = fields.values()?;
    let additions = tags(add_tags)?;
    let removals = tags(remove_tags)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let keys = normalized_existing_keys(&transaction, paths)?;
    for key in &keys {
        apply_fields(&transaction, key, &values)?;
        apply_tags(&transaction, key, &additions, &removals)?;
        super::search_index::rebuild(&transaction, key)?;
    }
    transaction.commit().map_err(|error| error.to_string())
}

fn validate_paths(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() || paths.len() > 500 {
        return Err("invalid_library_metadata_selection".into());
    }
    Ok(())
}

fn normalized_existing_keys(
    transaction: &Transaction<'_>,
    paths: &[String],
) -> Result<Vec<String>, String> {
    let mut keys = Vec::with_capacity(paths.len());
    let mut unique = HashSet::new();
    for path in paths {
        let key = db::normalize_key(path);
        if !unique.insert(key.clone()) {
            continue;
        }
        let exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM library_track lt
                 JOIN library_root lr ON lr.id=lt.root_id
                 WHERE lt.path_key=?1 AND lt.present=1 AND lr.enabled=1)",
                params![key],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if !exists {
            return Err("library_track_not_found".into());
        }
        keys.push(key);
    }
    Ok(keys)
}

fn apply_fields(
    transaction: &Transaction<'_>,
    key: &str,
    values: &[(&'static str, String)],
) -> Result<(), String> {
    if values.is_empty() {
        return Ok(());
    }
    transaction
        .execute(
            "INSERT OR IGNORE INTO track_user_metadata(path_key,updated_at)
             VALUES(?1,strftime('%s','now'))",
            params![key],
        )
        .map_err(|error| error.to_string())?;
    for (field, value) in values {
        if !FIELD_NAMES.contains(field) {
            return Err("invalid_library_metadata_field".into());
        }
        let sql = format!(
            "UPDATE track_user_metadata SET {field}=?2,updated_at=strftime('%s','now')
             WHERE path_key=?1"
        );
        transaction
            .execute(&sql, params![key, value])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn apply_tags(
    transaction: &Transaction<'_>,
    key: &str,
    additions: &[(String, String)],
    removals: &[(String, String)],
) -> Result<(), String> {
    for (_, normalized) in removals {
        transaction
            .execute(
                "DELETE FROM track_keyword WHERE path_key=?1 AND keyword_key=?2",
                params![key, normalized],
            )
            .map_err(|error| error.to_string())?;
    }
    let next: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(position),-1)+1 FROM track_keyword WHERE path_key=?1",
            params![key],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    for (offset, (display, normalized)) in additions.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO track_keyword(path_key,keyword,keyword_key,position)
                 VALUES(?1,?2,?3,?4)
                 ON CONFLICT(path_key,keyword_key) DO NOTHING",
                params![key, display, normalized, next + offset as i64],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn tags(values: &[String]) -> Result<Vec<(String, String)>, String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for value in values {
        let Some(display) = clean_tag(value)? else {
            continue;
        };
        let key = super::search_text::normalize(&display);
        if !key.is_empty() && seen.insert(key.clone()) {
            result.push((display, key));
        }
    }
    Ok(result)
}
