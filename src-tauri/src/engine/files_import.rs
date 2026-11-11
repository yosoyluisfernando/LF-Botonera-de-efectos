//! Copia y validación de documentos Android en almacenamiento privado.
use super::app;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri_plugin_dialog::FilePath;
use tauri_plugin_fs::{FsExt, OpenOptions};
pub(super) fn stamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
pub(super) fn safe_name(name: &str) -> String {
    let value: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "/\\:*?\"<>|".contains(c) {
                '_'
            } else {
                c
            }
        })
        .take(120)
        .collect();
    if value.is_empty() || value == "." || value == ".." {
        "document".into()
    } else {
        value
    }
}
pub(super) fn import(uri: FilePath, audio: bool) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    let dir = crate::core::platform::data_dir().join("imports");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let partial = dir.join(format!("{}.partial", stamp()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.read(true);
        let mut source = app()
            .fs()
            .open(uri.clone(), options)
            .map_err(|e| e.to_string())?;
        let mut destination = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&partial)
            .map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            let n = source.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            destination
                .write_all(&buffer[..n])
                .map_err(|e| e.to_string())?;
            digest.update(&buffer[..n]);
        }
        destination.sync_all().map_err(|e| e.to_string())?;
        drop(destination);
        if audio && !crate::engine::audio::decode::can_decode(&partial.to_string_lossy()) {
            return Err("unsupported_audio_format".into());
        }
        let raw = super::locutions::document_name(&uri)?;
        let mut name = safe_name(&raw);
        if audio && !crate::engine::audio::formats::is_audio_path(Path::new(&name)) {
            name.push_str(".wav"); // El decodificador identifica el formato por contenido.
        }
        let folder = dir.join(format!("{:x}", digest.finalize()));
        fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let final_path = folder.join(name);
        if final_path.exists() {
            fs::remove_file(&partial)
        } else {
            fs::rename(&partial, &final_path)
        }
        .map_err(|e| e.to_string())?;
        Ok(final_path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(partial);
    }
    result
}
