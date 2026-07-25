use super::{metadata::LibraryMetadata, metadata_batch, scanner};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Default)]
struct FormatStats {
    files: usize,
    readable: usize,
    fallback: usize,
    errors: usize,
}

fn roots() -> Vec<PathBuf> {
    std::env::var("LF_LIBRARY_BENCH_ROOTS")
        .unwrap_or_default()
        .split('|')
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[test]
#[ignore = "requiere LF_LIBRARY_BENCH_ROOTS y abre archivos reales en solo lectura"]
fn real_parallel_metadata_profile() {
    let files = roots()
        .iter()
        .flat_map(|root| scanner::discover(root, &[]).files)
        .collect::<Vec<_>>();
    assert!(!files.is_empty(), "no se encontraron audios");
    let paths = files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let workers = std::env::var("LF_LIBRARY_BENCH_WORKERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(metadata_batch::recommended_workers);
    let start = Instant::now();
    let results = metadata_batch::read(&paths, workers);
    let elapsed = start.elapsed();
    let mut formats = BTreeMap::<String, FormatStats>::new();
    let mut tags = [0usize; 6];
    for (path, result) in paths.iter().zip(results) {
        let key = Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("?")
            .to_ascii_lowercase();
        let stats = formats.entry(key).or_default();
        stats.files += 1;
        match result {
            Ok(meta) => record_success(stats, &mut tags, &meta),
            Err(_) => stats.errors += 1,
        }
    }
    for (format, stats) in formats {
        println!(
            "format={format} files={} readable={} fallback={} errors={}",
            stats.files, stats.readable, stats.fallback, stats.errors
        );
    }
    println!(
        "parallel workers={workers} files={} tags={tags:?} ms={}",
        paths.len(),
        elapsed.as_millis()
    );
}

#[test]
#[ignore = "requiere LF_LIBRARY_BENCH_ROOTS y repite lecturas reales en solo lectura"]
fn real_worker_comparison() {
    let paths = roots()
        .iter()
        .flat_map(|root| scanner::discover(root, &[]).files)
        .map(|file| file.path)
        .collect::<Vec<_>>();
    assert!(!paths.is_empty(), "no se encontraron audios");
    for workers in [1, 2, 4, 8, 4] {
        let start = Instant::now();
        let results = metadata_batch::read(&paths, workers);
        let readable = results.iter().filter(|result| result.is_ok()).count();
        let fallback = results
            .iter()
            .filter_map(|result| result.as_ref().ok())
            .filter(|meta| !meta.tags_readable)
            .count();
        println!(
            "workers={workers} files={} readable={readable} fallback={fallback} ms={}",
            paths.len(),
            start.elapsed().as_millis()
        );
    }
}

fn record_success(stats: &mut FormatStats, tags: &mut [usize; 6], meta: &LibraryMetadata) {
    stats.readable += 1;
    stats.fallback += usize::from(!meta.tags_readable);
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
        tags[index] += usize::from(*present);
    }
}
