/// Módulo: track_store.rs
/// Propósito: CRUD por archivo sobre tracks.db. La UI nunca toca SQL: pasa por
/// estos métodos vía comandos IPC (cmd_tracks.rs, etapa E.c). Separa el "qué se
/// guarda" (types_track) del "cómo se guarda" (este módulo).
use crate::db;
use crate::model::track::TrackMeta;
use rusqlite::{params, Connection, OptionalExtension, Row};

/// Columnas en el orden que esperan `map_row` y los INSERT/SELECT.
const COLS: &str = "path, mtime, size, duration_s, sample_rate, channels, \
    cue_start_s, cue_end_s, gain_db, norm_enabled, norm_gain_db, \
    measured_peak_db, measured_lufs, analyzed_at, last_played";

/// Almacén de metadatos de pista. Envuelve una conexión SQLite.
pub struct TrackStore {
    conn: Connection,
}

impl TrackStore {
    /// Abre el almacén en disco; si falla, degrada a memoria para no tumbar la
    /// app (la edición sigue funcionando, sin persistir entre sesiones).
    pub fn open() -> Self {
        match db::open(Some(&db::db_path())) {
            Ok(conn) => Self { conn },
            Err(e) => {
                eprintln!("tracks.db no disponible, usando memoria: {e}");
                Self {
                    conn: db::open(None).expect("SQLite en memoria"),
                }
            }
        }
    }

    /// Devuelve la fila de un archivo, o None si nunca se editó/analizó.
    pub fn get(&self, path: &str) -> Result<Option<TrackMeta>, String> {
        let key = db::normalize_key(path);
        let sql = format!("SELECT {COLS} FROM track WHERE path = ?1");
        self.conn
            .query_row(&sql, params![key], map_row)
            .optional()
            .map_err(|e| e.to_string())
    }

    /// Inserta una fila nueva o refresca los datos técnicos/medidos
    /// PRESERVANDO las ediciones del usuario (cue, dB, normalización activa,
    /// last_played). Lo usa el análisis (E.c) tras decodificar el archivo.
    pub fn upsert(&self, meta: &TrackMeta) -> Result<(), String> {
        let key = db::normalize_key(&meta.path);
        let sql = format!(
            "INSERT INTO track ({COLS}) VALUES \
             (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15) \
             ON CONFLICT(path) DO UPDATE SET \
               mtime=excluded.mtime, size=excluded.size, \
               duration_s=excluded.duration_s, sample_rate=excluded.sample_rate, \
               channels=excluded.channels, norm_gain_db=excluded.norm_gain_db, \
               measured_peak_db=excluded.measured_peak_db, \
               measured_lufs=excluded.measured_lufs, analyzed_at=excluded.analyzed_at"
        );
        self.conn
            .execute(
                &sql,
                params![
                    key,
                    meta.mtime,
                    meta.size,
                    meta.duration_s,
                    meta.sample_rate,
                    meta.channels,
                    meta.cue_start_s,
                    meta.cue_end_s,
                    meta.gain_db,
                    meta.norm_enabled as i64,
                    meta.norm_gain_db,
                    meta.measured_peak_db,
                    meta.measured_lufs,
                    meta.analyzed_at,
                    meta.last_played,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Persiste el cue (inicio/fin) editado por el usuario.
    pub fn set_cue(&self, path: &str, start_s: f64, end_s: Option<f64>) -> Result<(), String> {
        self.update(path, "cue_start_s=?2, cue_end_s=?3", params![db::normalize_key(path), start_s, end_s])
    }

    /// Persiste el trim manual en dB.
    pub fn set_gain(&self, path: &str, gain_db: f64) -> Result<(), String> {
        self.update(path, "gain_db=?2", params![db::normalize_key(path), gain_db])
    }

    /// Activa/desactiva la normalización automática para este archivo.
    pub fn set_normalization(&self, path: &str, enabled: bool) -> Result<(), String> {
        self.update(path, "norm_enabled=?2", params![db::normalize_key(path), enabled as i64])
    }

    /// Marca la última reproducción (epoch). Historial para la precarga.
    pub fn touch_last_played(&self, path: &str, epoch: i64) -> Result<(), String> {
        self.update(path, "last_played=?2", params![db::normalize_key(path), epoch])
    }

    /// Rutas recientes (>= since) y cortas (< max_dur), para el recalentado OnPlay.
    pub fn recent_paths(&self, since: i64, max_dur: f64) -> Result<Vec<String>, String> {
        let sql = "SELECT path FROM track WHERE last_played>=?1 AND duration_s>0 AND duration_s<?2";
        let mut st = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = st
            .query_map(params![since, max_dur], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// UPDATE acotado por clave (no falla si la fila aún no existe).
    fn update(&self, _path: &str, set: &str, p: &[&dyn rusqlite::ToSql]) -> Result<(), String> {
        let sql = format!("UPDATE track SET {set} WHERE path = ?1");
        self.conn.execute(&sql, p).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Convierte una fila SQL en TrackMeta (orden = COLS).
fn map_row(row: &Row) -> rusqlite::Result<TrackMeta> {
    Ok(TrackMeta {
        path: row.get(0)?,
        mtime: row.get(1)?,
        size: row.get(2)?,
        duration_s: row.get(3)?,
        sample_rate: row.get(4)?,
        channels: row.get(5)?,
        cue_start_s: row.get(6)?,
        cue_end_s: row.get(7)?,
        gain_db: row.get(8)?,
        norm_enabled: row.get::<_, i64>(9)? != 0,
        norm_gain_db: row.get(10)?,
        measured_peak_db: row.get(11)?,
        measured_lufs: row.get(12)?,
        analyzed_at: row.get(13)?,
        last_played: row.get(14)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_store() -> TrackStore {
        TrackStore {
            conn: db::open(None).unwrap(),
        }
    }

    #[test]
    fn upsert_then_get_roundtrip() {
        let store = memory_store();
        assert!(store.get("X:/no.mp3").unwrap().is_none()); // inexistente = None
        let meta = TrackMeta::new("C:/a/Risa.mp3".into(), 100, 2048, 3.5, 48000, 2);
        store.upsert(&meta).unwrap();
        let got = store.get("C:/a/Risa.mp3").unwrap().unwrap();
        assert_eq!(got.duration_s, 3.5);
        assert_eq!(got.channels, 2);
        assert_eq!(got.cue_start_s, 0.0);
    }

    #[test]
    fn user_edits_survive_reanalysis() {
        let store = memory_store();
        let path = "C:/a/Risa.mp3";
        store
            .upsert(&TrackMeta::new(path.into(), 100, 2048, 3.5, 48000, 2))
            .unwrap();
        // El usuario fija cue y dB.
        store.set_cue(path, 0.8, Some(3.0)).unwrap();
        store.set_gain(path, -2.5).unwrap();
        store.set_normalization(path, true).unwrap();
        // Re-análisis del MISMO archivo (mtime/duración nuevos) no pisa ediciones.
        store
            .upsert(&TrackMeta::new(path.into(), 200, 4096, 4.0, 44100, 1))
            .unwrap();
        let got = store.get(path).unwrap().unwrap();
        assert_eq!(got.cue_start_s, 0.8); // preservado
        assert_eq!(got.gain_db, -2.5); // preservado
        assert!(got.norm_enabled); // preservado
        assert_eq!(got.duration_s, 4.0); // refrescado
    }

    #[test]
    fn touch_last_played_and_recent_query() {
        let store = memory_store();
        let path = "C:/a/Risa.mp3";
        store
            .upsert(&TrackMeta::new(path.into(), 1, 1, 2.0, 48000, 2))
            .unwrap();
        store.touch_last_played(path, 1700000000).unwrap();
        assert_eq!(store.get(path).unwrap().unwrap().last_played, Some(1700000000));
        // Reciente (since menor) y corto → aparece; ventana futura → no.
        assert_eq!(store.recent_paths(1699999999, 30.0).unwrap().len(), 1);
        assert!(store.recent_paths(1700000001, 30.0).unwrap().is_empty());
    }
}
