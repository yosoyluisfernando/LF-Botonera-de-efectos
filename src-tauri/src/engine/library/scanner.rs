use crate::engine::audio::formats::{stamp_from_metadata, visit_audio_files, AudioWalkStats};
use crate::engine::persist::db::normalize_key;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredAudio {
    pub path: String,
    pub path_key: String,
    pub mtime: i64,
    pub size: i64,
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub files: Vec<DiscoveredAudio>,
    pub walk: AudioWalkStats,
    pub metadata_errors: usize,
}

/// Descubre archivos sin abrir su contenido. `excluded_roots` permite que una
/// raiz mas especifica de otra categoria gane sin producir filas duplicadas.
pub fn discover(root: &Path, excluded_roots: &[PathBuf]) -> Discovery {
    let mut discovery = Discovery::default();
    discovery.walk = visit_audio_files(
        root,
        |dir| !excluded_roots.iter().any(|excluded| dir == excluded),
        |path| match std::fs::metadata(path) {
            Ok(metadata) => {
                let text = path.to_string_lossy().to_string();
                let (mtime, size) = stamp_from_metadata(&metadata);
                discovery.files.push(DiscoveredAudio {
                    path_key: normalize_key(&text),
                    path: text,
                    mtime,
                    size,
                });
            }
            Err(_) => discovery.metadata_errors += 1,
        },
    );
    discovery
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn excludes_nested_root_without_duplicates() {
        let root = std::env::temp_dir().join(format!("lf_discover_{}", std::process::id()));
        let nested = root.join("effects");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("music.mp3"), b"x").unwrap();
        fs::write(nested.join("effect.wav"), b"x").unwrap();

        let found = discover(&root, std::slice::from_ref(&nested));
        let _ = fs::remove_dir_all(&root);

        assert_eq!(found.files.len(), 1);
        assert!(found.files[0].path.ends_with("music.mp3"));
    }
}
