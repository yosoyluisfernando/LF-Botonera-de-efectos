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
