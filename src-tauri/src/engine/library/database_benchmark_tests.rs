//! Perfiles manuales de la base nueva. Nunca abren `tracks.db` de la aplicación.
use super::{search, service::LibraryService};
use crate::engine::persist::db;
use rusqlite::{params, Connection};
use std::fs;
use std::time::Instant;

fn roots(variable: &str) -> Vec<String> {
    std::env::var(variable)
        .unwrap_or_default()
        .split('|')
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
#[ignore = "requiere las variables LF_LIBRARY_BENCH_*_ROOTS"]
fn real_roots_use_disposable_database() {
    let music = roots("LF_LIBRARY_BENCH_MUSIC_ROOTS");
    let effects = roots("LF_LIBRARY_BENCH_EFFECT_ROOTS");
    assert!(!music.is_empty() && !effects.is_empty(), "faltan raíces");
    let directory = super::test_support::tree("real_db");
    let database = directory.join("library-benchmark.sqlite");
    let service = LibraryService::new(database);
    for root in music {
        service.add_root(&root, "music").unwrap();
    }
    for root in effects {
        service.add_root(&root, "effects").unwrap();
    }
    for round in 1..=3 {
        let started = Instant::now();
        let reports = service.sync_all(|_| {}).unwrap();
        let discovered: usize = reports.iter().map(|report| report.discovered).sum();
        let enriched: usize = reports.iter().map(|report| report.enriched).sum();
        let unchanged: usize = reports.iter().map(|report| report.unchanged).sum();
        let failed: usize = reports.iter().map(|report| report.failed).sum();
        println!(
            "real_db round={round} discovered={discovered} enriched={enriched} \
             unchanged={unchanged} failed={failed} ms={}",
            started.elapsed().as_millis()
        );
    }
    println!("real_db status={:?}", service.status().unwrap());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "inserta 100000 y 250000 pistas sintéticas"]
fn synthetic_search_scale() {
    for count in [100_000usize, 250_000] {
        let mut connection = db::open(None).unwrap();
        let build_ms = populate(&mut connection, count);
        let mut elapsed = Vec::new();
        for _ in 0..20 {
            let started = Instant::now();
            let found =
                search::search(&connection, &format!("catalgoo {:06}", count - 1), None, 25)
                    .unwrap();
            assert!(!found.is_empty());
            elapsed.push(started.elapsed().as_micros());
        }
        elapsed.sort_unstable();
        println!(
            "synthetic rows={count} build_ms={build_ms} search_avg_us={} search_p95_us={}",
            elapsed.iter().sum::<u128>() / elapsed.len() as u128,
            elapsed[18]
        );
    }
}

fn populate(connection: &mut Connection, count: usize) -> u128 {
    let started = Instant::now();
    let transaction = connection.transaction().unwrap();
    transaction
        .execute(
            "INSERT INTO library_root(path,path_key,collection,created_at)
             VALUES('X:\\Biblioteca','x:\\biblioteca','music',0)",
            [],
        )
        .unwrap();
    for index in 0..count {
        insert_row(&transaction, index);
    }
    transaction.commit().unwrap();
    started.elapsed().as_millis()
}

fn insert_row(connection: &Connection, index: usize) {
    let key = format!("x:\\biblioteca\\tema_{index:06}.mp3");
    let name = format!("Tema Catálogo {index:06}.mp3");
    let searchable = format!("tema catalogo {index:06} artista {}", index % 1_000);
    connection
        .execute(
            "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
             VALUES(?1,1,1,180,44100,2)",
            params![key],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO library_track(path_key,root_id,collection,relative_path,file_name,
             extension,title,artist,metadata_state,tags_readable,last_seen_at)
             VALUES(?1,1,'music',?2,?2,'mp3',?2,?3,'ready',1,0)",
            params![key, name, format!("Artista {}", index % 1_000)],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO library_track_search(path_key,search_text) VALUES(?1,?2)",
            params![key, searchable],
        )
        .unwrap();
}
