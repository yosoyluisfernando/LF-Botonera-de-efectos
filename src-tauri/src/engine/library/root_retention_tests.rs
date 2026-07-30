use super::*;
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::library::test_support::{tree, write_wav};
use crate::engine::library::{indexer, root_store, search};
use crate::engine::persist::db;
use rusqlite::params;
use std::collections::HashSet;
use std::fs;

fn indexed_root(conn: &mut Connection, name: &str) -> (std::path::PathBuf, i64) {
    let root = tree(name);
    write_wav(&root.join("protegida.wav"), 1);
    write_wav(&root.join("huerfana.wav"), 1);
    let root_id = match root_store::add(conn, &root, LibraryCollection::Effects).unwrap() {
        root_store::AddOutcome::Added { root_id, .. } => root_id,
        other => panic!("alta inesperada: {other:?}"),
    };
    indexer::sync_root(conn, root_id, |_| {}).unwrap();
    (root, root_id)
}

#[test]
fn retention_is_limited_to_thirty_through_365_days() {
    let conn = db::open(None).unwrap();
    assert_eq!(settings(&conn).unwrap().retention_days, 30);
    assert_eq!(
        set_retention_days(&conn, 29).unwrap_err(),
        "library_retention_days_out_of_range"
    );
    assert_eq!(set_retention_days(&conn, 365).unwrap().retention_days, 365);
    assert!(set_retention_days(&conn, 366).is_err());
}

#[test]
fn retirement_hides_root_and_restore_reuses_catalog() {
    let mut conn = db::open(None).unwrap();
    let (root, root_id) = indexed_root(&mut conn, "retire_restore");
    let retired = retire_at(&conn, root_id, 1_000).unwrap();

    assert_eq!(retired.retired_at, 1_000);
    assert_eq!(retired.purge_after, 1_000 + 30 * DAY_SECONDS);
    assert!(root_store::list(&conn).unwrap().is_empty());
    assert!(search::search(&conn, "protegida", None, 10)
        .unwrap()
        .is_empty());
    assert_eq!(list(&conn).unwrap().len(), 1);

    root_store::restore(&mut conn, root_id).unwrap();

    assert_eq!(root_store::list(&conn).unwrap().len(), 1);
    assert_eq!(
        search::search(&conn, "protegida", None, 10).unwrap().len(),
        1
    );
    assert!(list(&conn).unwrap().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn changing_policy_does_not_shorten_an_existing_deadline() {
    let mut conn = db::open(None).unwrap();
    let (root, root_id) = indexed_root(&mut conn, "fixed_deadline");
    let deadline = retire_at(&conn, root_id, 50).unwrap().purge_after;

    set_retention_days(&conn, 365).unwrap();

    assert_eq!(list(&conn).unwrap()[0].purge_after, deadline);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn simulated_expiry_purges_orphans_but_preserves_referenced_track_metadata() {
    let mut conn = db::open(None).unwrap();
    let (root, root_id) = indexed_root(&mut conn, "purge");
    let protected_key: String = conn
        .query_row(
            "SELECT path_key FROM library_track WHERE file_name='protegida.wav'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let orphan_key: String = conn
        .query_row(
            "SELECT path_key FROM library_track WHERE file_name='huerfana.wav'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE track SET cue_start_s=0.25,gain_db=3 WHERE path=?1",
        params![&protected_key],
    )
    .unwrap();
    let deadline = retire_at(&conn, root_id, 100).unwrap().purge_after;
    let protected = HashSet::from([protected_key.clone()]);

    assert_eq!(
        purge_expired_at(&mut conn, &protected, deadline - 1)
            .unwrap()
            .roots,
        0
    );
    let report = purge_expired_at(&mut conn, &protected, deadline).unwrap();

    assert_eq!(report.roots, 1);
    assert_eq!(report.catalog_tracks, 2);
    assert_eq!(report.deleted_track_meta, 1);
    assert_eq!(report.protected_track_meta, 1);
    assert_eq!(
        conn.query_row(
            "SELECT cue_start_s FROM track WHERE path=?1",
            params![protected_key],
            |row| row.get::<_, f64>(0),
        )
        .unwrap(),
        0.25
    );
    assert!(!conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM track WHERE path=?1)",
            params![orphan_key],
            |row| row.get::<_, bool>(0),
        )
        .unwrap());
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM library_track_search", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        0
    );
    let _ = fs::remove_dir_all(root);
}
