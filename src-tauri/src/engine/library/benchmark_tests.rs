//! Pruebas manuales de rendimiento. Las rutas llegan por variable de entorno:
//! nunca se guardan ubicaciones ni nombres de archivos del usuario.
use super::{metadata, scanner};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

fn roots() -> Vec<PathBuf> {
    std::env::var("LF_LIBRARY_BENCH_ROOTS")
        .unwrap_or_default()
        .split('|')
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn discovered() -> Vec<scanner::DiscoveredAudio> {
    roots()
        .iter()
        .flat_map(|root| scanner::discover(root, &[]).files)
        .collect()
}

#[test]
#[ignore = "requiere LF_LIBRARY_BENCH_ROOTS y recorre archivos reales en solo lectura"]
fn real_discovery_profile() {
    let roots = roots();
    assert!(!roots.is_empty(), "falta LF_LIBRARY_BENCH_ROOTS");
    let rounds = std::env::var("LF_LIBRARY_BENCH_ROUNDS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);
    for round in 1..=rounds {
        let start = Instant::now();
        let mut files = 0;
        let mut directories = 0;
        let mut errors = 0;
        for root in &roots {
            let found = scanner::discover(root, &[]);
            files += found.files.len();
            directories += found.walk.directories;
            errors += found.walk.inaccessible + found.metadata_errors;
        }
        println!(
            "discovery round={round} files={files} dirs={directories} errors={errors} ms={}",
            start.elapsed().as_millis()
        );
    }
}

#[test]
#[ignore = "requiere LF_LIBRARY_BENCH_ROOTS y abre archivos reales en solo lectura"]
fn real_metadata_profile() {
    let files = discovered();
    assert!(!files.is_empty(), "no se encontraron audios");
    let start = Instant::now();
    let mut readable = 0;
    let mut fallback = 0;
    let mut errors = 0;
    let mut tag_fields = [0usize; 6];
    let mut formats = HashMap::<String, usize>::new();
    for file in &files {
        let extension = PathBuf::from(&file.path)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("?")
            .to_ascii_lowercase();
        *formats.entry(extension).or_default() += 1;
        match metadata::read(&file.path) {
            Ok(meta) => {
                readable += 1;
                fallback += usize::from(!meta.tags_readable);
                for (index, present) in [
                    meta.title.is_some(),
                    meta.artist.is_some(),
                    meta.album.is_some(),
                    meta.genre.is_some(),
                    meta.year.is_some(),
                    meta.track.is_some(),
                ]
                .iter()
                .enumerate()
                {
                    tag_fields[index] += usize::from(*present);
                }
            }
            Err(_) => errors += 1,
        }
    }
    println!(
        "metadata files={} readable={readable} fallback={fallback} errors={errors} tags={tag_fields:?} formats={formats:?} ms={}",
        files.len(),
        start.elapsed().as_millis()
    );
}

#[test]
#[ignore = "requiere LF_LIBRARY_BENCH_ROOTS; simula una reconciliacion sin cambios"]
fn real_unchanged_profile() {
    let first = discovered();
    let known: HashSet<_> = first
        .iter()
        .map(|file| (file.path_key.clone(), file.mtime, file.size))
        .collect();
    let start = Instant::now();
    let second = discovered();
    let changed = second
        .iter()
        .filter(|file| !known.contains(&(file.path_key.clone(), file.mtime, file.size)))
        .count();
    println!(
        "unchanged files={} changed={changed} ms={}",
        second.len(),
        start.elapsed().as_millis()
    );
}
