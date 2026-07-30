//! Validación portable del nombre físico elegido por el usuario.
use crate::engine::persist::db;
use std::path::{Path, PathBuf};

pub fn destination(source: &Path, requested: &str) -> Result<PathBuf, String> {
    let name = requested.trim();
    if name.is_empty() || name == "." || name == ".." || name.as_bytes().len() > 255 {
        return Err("library_rename_invalid_name".into());
    }
    if name.chars().any(invalid_character)
        || name.ends_with('.')
        || name.ends_with(' ')
        || reserved_windows_name(name)
    {
        return Err("library_rename_invalid_name".into());
    }
    let current = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("library_rename_invalid_source")?;
    if current == name {
        return Err("library_rename_unchanged".into());
    }
    let old_extension = source.extension().and_then(|value| value.to_str());
    let new_extension = Path::new(name).extension().and_then(|value| value.to_str());
    if old_extension != new_extension {
        return Err("library_rename_extension_changed".into());
    }
    let parent = source.parent().ok_or("library_rename_invalid_source")?;
    let target = parent.join(name);
    if target.exists()
        && db::normalize_key(&target.to_string_lossy())
            != db::normalize_key(&source.to_string_lossy())
    {
        return Err("library_rename_destination_exists".into());
    }
    Ok(target)
}

pub fn temporary_sibling(source: &Path) -> Result<PathBuf, String> {
    let parent = source.parent().ok_or("library_rename_invalid_source")?;
    for attempt in 0..1000 {
        let candidate = parent.join(format!(".lf-rename-{}-{attempt}.tmp", std::process::id()));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("library_rename_temporary_unavailable".into())
}

fn invalid_character(value: char) -> bool {
    value.is_control() || matches!(value, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
}

fn reserved_windows_name(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && matches!(&stem[..3], "COM" | "LPT")
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_directory_and_extension() {
        let result = destination(Path::new("C:/audio/risa.mp3"), "Risa con aplausos.mp3").unwrap();
        assert_eq!(result, PathBuf::from("C:/audio/Risa con aplausos.mp3"));
    }

    #[test]
    fn rejects_reserved_invalid_and_extension_changes() {
        let source = Path::new("C:/audio/risa.mp3");
        for name in ["CON.mp3", "mala?.mp3", "../otra.mp3", "risa.wav"] {
            assert!(destination(source, name).is_err(), "{name}");
        }
    }
}
