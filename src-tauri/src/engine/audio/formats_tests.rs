//! Pruebas del recorrido de carpetas. Se crea un arbol al vuelo: no dependen de
//! ningun archivo del equipo.
use super::*;
use std::fs;

fn tree(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("lf_scan_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("sub/hondo")).unwrap();
    fs::write(root.join("a.mp3"), b"x").unwrap();
    fs::write(root.join("notas.txt"), b"x").unwrap(); // no es audio
    fs::write(root.join("sub/b.flac"), b"x").unwrap();
    fs::write(root.join("sub/hondo/c.wav"), b"x").unwrap();
    root
}

/// Debe entrar en las subcarpetas: es lo que se pidio al soltar una carpeta.
#[test]
fn finds_audio_in_subfolders() {
    let root = tree("sub");
    let found = audio_files_recursive(root.to_str().unwrap());
    let _ = fs::remove_dir_all(&root);

    assert_eq!(found.len(), 3, "las tres, incluidas las de subcarpetas");
    assert!(found.iter().any(|f| f.ends_with("c.wav")), "hasta el nivel mas hondo");
}

/// Lo que no es audio no entra, y el orden es estable (alfabetico por ruta).
#[test]
fn skips_non_audio_and_sorts() {
    let root = tree("sort");
    let found = audio_files_recursive(root.to_str().unwrap());
    let _ = fs::remove_dir_all(&root);

    assert!(!found.iter().any(|f| f.ends_with(".txt")));
    let mut sorted = found.clone();
    sorted.sort();
    assert_eq!(found, sorted, "el orden no puede depender del sistema de archivos");
}

/// Una carpeta que no existe no revienta: simplemente no hay audio.
#[test]
fn a_missing_folder_is_empty_not_a_panic() {
    assert!(audio_files_recursive(r"C:\no\existe\nada").is_empty());
}
