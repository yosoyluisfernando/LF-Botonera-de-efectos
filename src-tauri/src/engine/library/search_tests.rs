use super::*;
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::library::indexer;
use crate::engine::library::test_support::{add_root, tree, write_wav};
use crate::engine::persist::db;
use std::fs;
use std::path::PathBuf;

fn catalog() -> (Connection, PathBuf) {
    let root = tree("search");
    let effects = root.join("effects");
    fs::create_dir_all(&effects).unwrap();
    write_wav(&root.join("Canción Eterna.wav"), 2);
    write_wav(&effects.join("Sirena Radio.wav"), 1);
    let mut conn = db::open(None).unwrap();
    let music_id = add_root(&mut conn, &root, LibraryCollection::Music);
    let effects_id = add_root(&mut conn, &effects, LibraryCollection::Effects);
    indexer::sync_root(&mut conn, music_id, |_| {}).unwrap();
    indexer::sync_root(&mut conn, effects_id, |_| {}).unwrap();
    (conn, root)
}

#[test]
fn accentless_exact_query_finds_filename() {
    let (conn, root) = catalog();
    let results = search(&conn, "cancion eterna", None, 10).unwrap();
    let _ = fs::remove_dir_all(root);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].collection, "music");
}

#[test]
fn typo_and_transposition_are_tolerated() {
    let (conn, root) = catalog();
    let song = search(&conn, "cansion", None, 10).unwrap();
    let effect = search(&conn, "sirnea", None, 10).unwrap();
    let _ = fs::remove_dir_all(root);
    assert_eq!(song[0].file_name, "Canción Eterna.wav");
    assert_eq!(effect[0].file_name, "Sirena Radio.wav");
}

#[test]
fn category_filter_and_unrelated_query_are_strict() {
    let (conn, root) = catalog();
    assert!(search(&conn, "sirena", Some("music"), 10)
        .unwrap()
        .is_empty());
    assert_eq!(
        search(&conn, "sirena", Some("effects"), 10).unwrap().len(),
        1
    );
    assert!(search(&conn, "helicoptero", None, 10).unwrap().is_empty());
    let _ = fs::remove_dir_all(root);
}
