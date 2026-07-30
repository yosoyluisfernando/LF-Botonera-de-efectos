//! Hijos de carpeta derivados del catálogo; no vuelve a recorrer el disco.
use super::service::LibraryService;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize)]
pub struct CatalogFolder {
    pub name: String,
    pub relative_path: String,
}

impl LibraryService {
    pub fn folder_children(
        &self,
        root_id: i64,
        parent: Option<&str>,
    ) -> Result<Vec<CatalogFolder>, String> {
        read(&self.connection()?, root_id, parent.unwrap_or(""))
    }
}

fn read(connection: &Connection, root_id: i64, parent: &str) -> Result<Vec<CatalogFolder>, String> {
    let parent = normalize(parent);
    let prefix = if parent.is_empty() {
        String::new()
    } else {
        format!("{parent}/")
    };
    let pattern = format!("{}%", escape_like(&prefix));
    let mut statement = connection
        .prepare(
            "SELECT replace(lt.relative_path,'\\','/')
             FROM library_track lt
             JOIN library_root lr ON lr.id=lt.root_id
             WHERE lt.root_id=?1 AND lt.present=1 AND lr.enabled=1
             AND replace(lt.relative_path,'\\','/') LIKE ?2 ESCAPE '\\'",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![root_id, pattern], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    let mut names = BTreeSet::new();
    for path in rows {
        let path = path.map_err(|error| error.to_string())?;
        let remainder = path.strip_prefix(&prefix).unwrap_or(&path);
        if let Some((folder, _)) = remainder.split_once('/') {
            if !folder.is_empty() {
                names.insert(folder.to_string());
            }
        }
    }
    Ok(names
        .into_iter()
        .map(|name| CatalogFolder {
            relative_path: if parent.is_empty() {
                name.clone()
            } else {
                format!("{parent}/{name}")
            },
            name,
        })
        .collect())
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/").trim_matches('/').to_string()
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_native_separators() {
        assert_eq!(normalize(r"Rock\Clásicos\"), "Rock/Clásicos");
    }
}
