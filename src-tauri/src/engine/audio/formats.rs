use crate::engine::audio::decode as audio_decode;
/// Modulo: audio_formats.rs
/// Proposito: lista unica de extensiones y validacion de archivos de audio.
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

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
/// Duracion en segundos, o -1.0 si no se puede leer.
///
/// **Se piden las propiedades SIN las etiquetas** (`read_tags(false)`). Solo
/// necesitamos la duracion, y leer las etiquetas de paso hacia perder el archivo
/// entero: un MP3 con el titulo o el artista mal codificados (frecuente en
/// rippeos viejos con acentos) devolvia `TextDecode("Found invalid encoding")` y
/// con el la duracion, que no tiene nada que ver con el texto. El audio estaba
/// perfecto; solo sobraba leer lo que no usamos.
pub fn probe_duration_secs(path: &str) -> f64 {
    use lofty::config::ParseOptions;
    use lofty::file::AudioFile;
    use lofty::probe::Probe;
    Probe::open(path)
        .and_then(|probe| probe.options(ParseOptions::new().read_tags(false)).read())
        .map(|f| f.properties().duration().as_secs_f64())
        .unwrap_or(-1.0)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AudioWalkStats {
    pub directories: usize,
    pub audio_files: usize,
    pub inaccessible: usize,
}

/// Recorre una raiz una sola vez y entrega cada audio al consumidor.
/// Es la fuente unica para importar carpetas y para indexar la biblioteca.
pub fn visit_audio_files<F, D>(root: &Path, mut descend: D, mut visit: F) -> AudioWalkStats
where
    F: FnMut(&Path),
    D: FnMut(&Path) -> bool,
{
    let mut stats = AudioWalkStats::default();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        if dir != root && !descend(&dir) {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            stats.inaccessible += 1;
            continue;
        };
        stats.directories += 1;
        for result in entries {
            let Ok(entry) = result else {
                stats.inaccessible += 1;
                continue;
            };
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(kind) if kind.is_file() && is_audio_path(&path) => {
                    stats.audio_files += 1;
                    visit(&path);
                }
                Err(_) => stats.inaccessible += 1,
                _ => {}
            }
        }
    }
    stats
}

/// Sello barato usado para decidir si un archivo necesita releerse.
pub fn stamp_from_metadata(metadata: &Metadata) -> (i64, i64) {
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    (mtime, metadata.len() as i64)
}

pub fn file_stamp(path: &Path) -> (i64, i64) {
    std::fs::metadata(path)
        .map(|metadata| stamp_from_metadata(&metadata))
        .unwrap_or((0, 0))
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
    visit_audio_files(
        &PathBuf::from(folder),
        |_| true,
        |path| out.push(path.to_string_lossy().to_string()),
    );
    out.sort();
    out
}

#[cfg(test)]
#[path = "formats_tests.rs"]
mod formats_tests;
