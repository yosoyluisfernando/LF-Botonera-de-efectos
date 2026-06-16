/// Módulo: locutions.rs
/// Propósito: Resolución de archivos de locución, traducida del LF
/// Automatizador (frontend/render.js):
///  - Hora:        HRS{hh} + MIN{mm}; al minuto 00 solo HRS{hh}_O ("en punto").
///  - Temperatura: TMP{###} (positiva) / TMPN{###} (negativa).
///  - Humedad:     HUM{###} (0-100).
/// Se busca el PRIMER archivo de la carpeta cuyo nombre empiece con el prefijo.
use std::fs;
use std::path::{Path, PathBuf};

/// Devuelve los archivos a reproducir en secuencia para anunciar la hora.
pub fn resolve_time_files(folder: &str) -> Result<Vec<String>, String> {
    let entries = read_folder(folder)?;
    let now = chrono_now();
    let (hh, mm) = (format!("{:02}", now.0), format!("{:02}", now.1));

    let mut files = Vec::new();
    if mm == "00" {
        // En punto: variante especial HRS{hh}_O
        if let Some(f) = find_prefix(&entries, &format!("HRS{}_O", hh), None) {
            files.push(f);
        }
    } else {
        // HRS{hh} (excluyendo la variante _O) seguido de MIN{mm}
        if let Some(f) = find_prefix(&entries, &format!("HRS{}", hh), Some("_O")) {
            files.push(f);
        }
        if let Some(f) = find_prefix(&entries, &format!("MIN{}", mm), None) {
            files.push(f);
        }
    }
    if files.is_empty() {
        return Err("No hay locuciones de hora para este momento".to_string());
    }
    Ok(files)
}

/// Devuelve el archivo de locución para un valor de clima.
/// `kind`: "temperature" | "humidity".
pub fn resolve_climate_file(folder: &str, kind: &str, value: f64) -> Result<String, String> {
    let entries = read_folder(folder)?;
    let rounded = value.round() as i64;
    let prefix = if kind == "humidity" {
        format!("HUM{:03}", rounded.clamp(0, 100))
    } else if rounded < 0 {
        format!("TMPN{:03}", rounded.abs())
    } else {
        format!("TMP{:03}", rounded)
    };
    find_prefix(&entries, &prefix, None).ok_or(format!("No hay locución para el valor {}", rounded))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Lista los archivos de una carpeta como rutas absolutas.
fn read_folder(folder: &str) -> Result<Vec<PathBuf>, String> {
    if folder.is_empty() || !Path::new(folder).exists() {
        return Err("Carpeta de locuciones no configurada o inexistente".to_string());
    }
    Ok(fs::read_dir(folder)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect())
}

/// Primer archivo cuyo nombre (en mayúsculas) empiece con `prefix`,
/// excluyendo opcionalmente los que contengan `exclude`.
fn find_prefix(entries: &[PathBuf], prefix: &str, exclude: Option<&str>) -> Option<String> {
    let prefix = prefix.to_uppercase();
    entries
        .iter()
        .find(|p| {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_uppercase())
                .unwrap_or_default();
            name.starts_with(&prefix) && exclude.map_or(true, |ex| !name.contains(ex))
        })
        .map(|p| p.to_string_lossy().to_string())
}

/// Hora y minuto locales del SO (chrono respeta zona horaria y DST).
fn chrono_now() -> (u32, u32) {
    use chrono::Timelike;
    let now = chrono::Local::now();
    (now.hour(), now.minute())
}
