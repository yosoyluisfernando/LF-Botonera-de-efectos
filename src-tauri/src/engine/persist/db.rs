/// Módulo: db.rs
/// Propósito: conexión SQLite (tracks.db) y migración del esquema por
/// `PRAGMA user_version`. Punto ÚNICO donde vive la diferencia de SO en la
/// clave de archivo (Windows no distingue mayúsculas; Linux sí).
use crate::engine::persist::config_io::get_data_dir;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Versión actual del esquema. Subir este número al añadir una migración.
const SCHEMA_VERSION: i64 = 1;

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
    if version != SCHEMA_VERSION {
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

const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS track (
  path             TEXT PRIMARY KEY,
  mtime            INTEGER NOT NULL,
  size             INTEGER NOT NULL,
  duration_s       REAL    NOT NULL,
  sample_rate      INTEGER NOT NULL,
  channels         INTEGER NOT NULL,
  cue_start_s      REAL    NOT NULL DEFAULT 0,
  cue_end_s        REAL,
  gain_db          REAL    NOT NULL DEFAULT 0,
  norm_enabled     INTEGER NOT NULL DEFAULT 0,
  norm_gain_db     REAL    NOT NULL DEFAULT 0,
  measured_peak_db REAL,
  measured_lufs    REAL,
  analyzed_at      INTEGER,
  last_played      INTEGER
);
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_sets_user_version_and_table() {
        let conn = open(None).unwrap();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        // La tabla debe existir y poder consultarse.
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM track", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = open(None).unwrap();
        // Volver a migrar no debe fallar ni duplicar nada.
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }

    #[test]
    fn normalize_key_is_consistent() {
        // En cualquier SO, normalizar dos veces da lo mismo (idempotente).
        let a = normalize_key("C:/Audio/Risa.mp3");
        let b = normalize_key(&a);
        assert_eq!(a, b);
    }
}
