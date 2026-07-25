use super::*;
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::library::test_support::{add_root, tree, write_wav};
use crate::engine::persist::db;
use std::fs;

#[test]
fn second_scan_skips_unchanged_metadata() {
    let root = tree("incremental");
    write_wav(&root.join("one.wav"), 1);
    let mut conn = db::open(None).unwrap();
    let root_id = add_root(&mut conn, &root, LibraryCollection::Music);

    let first = sync_root(&mut conn, root_id, |_| {}).unwrap();
    let second = sync_root(&mut conn, root_id, |_| {}).unwrap();
    let _ = fs::remove_dir_all(root);

    assert_eq!(first.enriched, 1);
    assert_eq!(second.enriched, 0);
    assert_eq!(second.unchanged, 1);
}

#[test]
fn missing_file_is_marked_without_deleting_track_edits() {
    let root = tree("missing");
    let path = root.join("one.wav");
    write_wav(&path, 1);
    let mut conn = db::open(None).unwrap();
    let root_id = add_root(&mut conn, &root, LibraryCollection::Effects);
    sync_root(&mut conn, root_id, |_| {}).unwrap();
    conn.execute(
        "UPDATE track SET gain_db=-3 WHERE path=(SELECT path_key FROM library_track LIMIT 1)",
        [],
    )
    .unwrap();
    fs::remove_file(path).unwrap();

    let report = sync_root(&mut conn, root_id, |_| {}).unwrap();
    let (present, gain): (i64, f64) = conn
        .query_row(
            "SELECT lt.present,t.gain_db FROM library_track lt
             JOIN track t ON t.path=lt.path_key LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let _ = fs::remove_dir_all(root);

    assert_eq!(report.missing, 1);
    assert_eq!(present, 0);
    assert_eq!(gain, -3.0);
}

#[test]
fn nested_category_owns_its_files_once() {
    let parent = tree("categories");
    let child = parent.join("effects");
    fs::create_dir_all(&child).unwrap();
    write_wav(&parent.join("song.wav"), 2);
    write_wav(&child.join("effect.wav"), 1);
    let mut conn = db::open(None).unwrap();
    let music_id = add_root(&mut conn, &parent, LibraryCollection::Music);
    let effects_id = add_root(&mut conn, &child, LibraryCollection::Effects);

    sync_root(&mut conn, music_id, |_| {}).unwrap();
    sync_root(&mut conn, effects_id, |_| {}).unwrap();
    let music: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM library_track WHERE collection='music'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM library_track WHERE collection='effects'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM library_track", [], |row| row.get(0))
        .unwrap();
    let _ = fs::remove_dir_all(parent);

    assert_eq!((music, effects, total), (1, 1, 2));
}
