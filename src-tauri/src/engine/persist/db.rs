/// Módulo: db.rs
/// Propósito: conexión SQLite (tracks.db) y migración del esquema por
/// `PRAGMA user_version`. Punto ÚNICO donde vive la diferencia de SO en la
/// clave de archivo (Windows no distingue mayúsculas; Linux sí).
use crate::engine::persist::config_io::get_data_dir;
use db_schema::{SCHEMA_V1, SCHEMA_V2, SCHEMA_V3, SCHEMA_V4, SCHEMA_V5, SCHEMA_V6};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[path = "db_schema.rs"]
mod db_schema;

/// Versión actual del esquema. Subir este número al añadir una migración.
pub const SCHEMA_VERSION: i64 = 6;

/// Ruta del fichero de base de datos, junto al config (multiplataforma).
pub fn db_path() -> PathBuf {
    get_data_dir().join("tracks.db")
}

/// Abre la base de datos con WAL y el esquema migrado.
/// `path = None` abre una base en memoria (tests / degradación sin persistir).
pub fn open(path: Option<&Path>) -> Result<Connection, String> {
    let conn = match path {
        Some(p) => {
            if let Some(dir) = p.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            Connection::open(p).map_err(|e| e.to_string())?
        }
        None => Connection::open_in_memory().map_err(|e| e.to_string())?,
    };
    // WAL = escrituras frecuentes baratas (last_played) sin bloqueos largos.
    // journal_mode devuelve una fila, por eso se consulta en vez de execute.
    let _: String = conn
        .query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
        .map_err(|e| e.to_string())?;
    migrate(&conn)?;
    Ok(conn)
}

/// Crea/actualiza el esquema según `PRAGMA user_version`.
pub fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if version < 1 {
        conn.execute_batch(SCHEMA_V1).map_err(|e| e.to_string())?;
    }
    if version < 2 {
        conn.execute_batch(SCHEMA_V2).map_err(|e| e.to_string())?;
    }
    if version < 3 {
        conn.execute_batch(SCHEMA_V3).map_err(|e| e.to_string())?;
    }
    if version < 4 {
        conn.execute_batch(SCHEMA_V4).map_err(|e| e.to_string())?;
    }
    if version < 5 {
        conn.execute_batch(SCHEMA_V5).map_err(|e| e.to_string())?;
    }
    if version < 6 {
        conn.execute_batch(SCHEMA_V6).map_err(|e| e.to_string())?;
    }
    if version > SCHEMA_VERSION {
        return Err("database_schema_newer".into());
    }
    if version < SCHEMA_VERSION {
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Normaliza la ruta para usarla como clave primaria. En Windows el sistema de
/// archivos no distingue mayúsculas → se compara en minúsculas; en Linux sí.
pub fn normalize_key(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        path.to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string()
    }
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod db_tests;
