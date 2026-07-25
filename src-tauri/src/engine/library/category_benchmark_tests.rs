use super::{metadata::LibraryMetadata, metadata_batch, scanner};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Default)]
struct CategoryStats {
    files: usize,
    readable: usize,
    fallback: usize,
    errors: usize,
    tags: [usize; 6],
    durations: Vec<f64>,
}

fn roots(variable: &str) -> Vec<PathBuf> {
    std::env::var(variable)
        .unwrap_or_default()
        .split('|')
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[test]
#[ignore = "requiere las dos variables LF_LIBRARY_BENCH_*_ROOTS"]
fn real_category_metadata_profile() {
    for (name, variable) in [
        ("music", "LF_LIBRARY_BENCH_MUSIC_ROOTS"),
        ("effects", "LF_LIBRARY_BENCH_EFFECT_ROOTS"),
    ] {
        let paths = roots(variable)
            .iter()
            .flat_map(|root| scanner::discover(root, &[]).files)
            .map(|file| file.path)
            .collect::<Vec<_>>();
        assert!(!paths.is_empty(), "falta {variable}");
        let start = Instant::now();
        let results = metadata_batch::read(&paths, 4);
        let mut stats = CategoryStats {
            files: paths.len(),
            ..CategoryStats::default()
        };
        for result in results {
            match result {
                Ok(metadata) => record(&mut stats, &metadata),
                Err(_) => stats.errors += 1,
            }
        }
        stats.durations.sort_by(f64::total_cmp);
        println!(
            "category={name} files={} readable={} fallback={} errors={} tags={:?} median_s={:.3} p95_s={:.3} max_s={:.3} ms={}",
            stats.files,
            stats.readable,
            stats.fallback,
            stats.errors,
            stats.tags,
            percentile(&stats.durations, 50),
            percentile(&stats.durations, 95),
            stats.durations.last().copied().unwrap_or(0.0),
            start.elapsed().as_millis()
        );
    }
}

fn record(stats: &mut CategoryStats, metadata: &LibraryMetadata) {
    stats.readable += 1;
    stats.fallback += usize::from(!metadata.tags_readable);
    if metadata.duration_s > 0.0 {
        stats.durations.push(metadata.duration_s);
    }
    for (index, present) in [
        metadata.title.is_some(),
        metadata.artist.is_some(),
        metadata.album.is_some(),
        metadata.genre.is_some(),
        metadata.year.is_some(),
        metadata.track.is_some(),
    ]
    .iter()
    .enumerate()
    {
        stats.tags[index] += usize::from(*present);
    }
}

fn percentile(sorted: &[f64], percent: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[(sorted.len() - 1) * percent / 100]
}
