/// Módulo: resolver.rs
/// Propósito: llevar a disco las reglas de `domain::locution`. Lee la carpeta,
/// mira el reloj y pregunta qué archivo toca; no decide nada.
///
/// Los errores salen como CLAVES, no como frases: el texto lo pone i18n (regla
/// 7). Una locución que falla en mitad de un directo tiene que poder explicarse
/// en el idioma del operador.
use crate::domain::locution;
use std::fs;
use std::path::{Path, PathBuf};

/// Los archivos a encadenar para anunciar la hora.
pub fn resolve_time_files(folder: &str) -> Result<Vec<String>, String> {
    let entries = read_folder(folder)?;
    let (hh, mm) = chrono_now();
    let sequence = locution::time_sequence(&names_of(&entries), hh, mm);
    if sequence.is_empty() {
        return Err("no_time_locution".to_string());
    }
    Ok(sequence.iter().map(|&i| path_of(&entries[i])).collect())
}

/// El archivo de locución para un valor de clima.
/// `kind`: "temperature" | "humidity".
pub fn resolve_climate_file(folder: &str, kind: &str, value: f64) -> Result<String, String> {
    let entries = read_folder(folder)?;
    locution::climate(&names_of(&entries), kind, value)
        .map(|i| path_of(&entries[i]))
        .ok_or_else(|| "no_climate_locution".to_string())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Lista los archivos de una carpeta como rutas absolutas.
fn read_folder(folder: &str) -> Result<Vec<PathBuf>, String> {
    if folder.is_empty() || !Path::new(folder).exists() {
        return Err("locution_folder_missing".to_string());
    }
    Ok(fs::read_dir(folder)
        .map_err(|_| "locution_folder_missing".to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect())
}

/// Los nombres sueltos, en el mismo orden que `entries`: el dominio decide por
/// nombre y no tiene por qué saber de rutas.
fn names_of(entries: &[PathBuf]) -> Vec<String> {
    entries
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        })
        .collect()
}

fn path_of(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

/// Hora y minuto locales del SO (chrono respeta zona horaria y DST).
fn chrono_now() -> (u32, u32) {
    use chrono::Timelike;
    let now = chrono::Local::now();
    (now.hour(), now.minute())
}
