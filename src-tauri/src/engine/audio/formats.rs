use crate::engine::audio::decode as audio_decode;
/// Modulo: audio_formats.rs
/// Proposito: lista unica de extensiones y validacion de archivos de audio.
use std::path::Path;

pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "opus", "aac", "m4a", "aiff", "wma",
];

/// Devuelve true si una extension pertenece a los formatos aceptados.
pub fn is_audio_extension(ext: &str) -> bool {
    AUDIO_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
}

/// Indica si una ruta tiene extension de audio aceptada.
pub fn is_audio_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(is_audio_extension)
        .unwrap_or(false)
}

/// Valida que una ruta exista como archivo y tenga extension de audio aceptada.
pub fn validate_audio_file(path: &str) -> Result<(), String> {
    let file = Path::new(path);
    if !file.is_file() {
        return Err("audio_file_not_found".to_string());
    }
    if !is_audio_path(file) {
        return Err("unsupported_audio_format".to_string());
    }
    if !audio_decode::can_decode(path) {
        return Err("unsupported_audio_format".to_string());
    }
    Ok(())
}

/// Sonda la duracion de un archivo leyendo sus propiedades, sin decodificarlo.
pub fn probe_duration_secs(path: &str) -> f64 {
    use lofty::file::AudioFile;
    lofty::read_from_path(path)
        .map(|f| f.properties().duration().as_secs_f64())
        .unwrap_or(-1.0)
}

/// Todos los audios de una carpeta y sus subcarpetas, en orden alfabetico por
/// ruta completa (asi cada subcarpeta queda agrupada y ordenada).
///
/// Solo mira las extensiones: listar es barato (medido, ~21 ms para 1.910
/// archivos en 53 carpetas), mientras que abrir cada archivo para saber su
/// duracion cuesta ~40 ms **por archivo**. Por eso quien llame a esto puede
/// contar de inmediato y dejar lo caro para despues.
pub fn audio_files_recursive(folder: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pending = vec![std::path::PathBuf::from(folder)];
    // Pila explicita en vez de recursion: un arbol muy anidado no debe poder
    // desbordar la pila del hilo.
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue; // una carpeta sin permisos no aborta el resto
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(t) if t.is_dir() => pending.push(path),
                Ok(t) if t.is_file() && is_audio_path(&path) => {
                    out.push(path.to_string_lossy().to_string());
                }
                _ => {}
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
#[path = "formats_tests.rs"]
mod formats_tests;
