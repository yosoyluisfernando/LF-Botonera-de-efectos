use super::*;
use crate::engine::library::test_support::{tree, write_wav};
use crate::engine::library::{indexer, search};
use crate::engine::persist::db;
use std::fs;

#[test]
fn multiple_independent_roots_can_share_collection() {
    let mut conn = db::open(None).unwrap();
    let first = tree("first");
    let second = tree("second");
    add(&mut conn, &first, LibraryCollection::Music).unwrap();
    add(&mut conn, &second, LibraryCollection::Music).unwrap();
    let roots = list(&conn).unwrap();
    let _ = fs::remove_dir_all(first);
    let _ = fs::remove_dir_all(second);

    assert_eq!(roots.len(), 2);
    assert!(roots.iter().all(|root| root.collection == "music"));
}

#[test]
fn batch_is_atomic_when_any_folder_is_invalid() {
    let mut conn = db::open(None).unwrap();
    let valid = tree("batch_atomic");
    let missing = valid.join("does-not-exist");
    let roots = vec![
        (valid.clone(), LibraryCollection::Music),
        (missing, LibraryCollection::Effects),
    ];

    assert!(add_batch(&mut conn, &roots).is_err());
    assert!(list(&conn).unwrap().is_empty());
    let _ = fs::remove_dir_all(valid);
}

#[test]
fn batch_unifies_nested_folders_without_duplicates() {
    let mut conn = db::open(None).unwrap();
    let parent = tree("batch_merge");
    let child = parent.join("child");
    fs::create_dir_all(&child).unwrap();
    let outcomes = add_batch(
        &mut conn,
        &[
            (child, LibraryCollection::Music),
            (parent.clone(), LibraryCollection::Music),
        ],
    )
    .unwrap();

    assert_eq!(outcomes.len(), 2);
    assert_eq!(list(&conn).unwrap().len(), 1);
    let _ = fs::remove_dir_all(parent);
}

#[test]
fn parent_merges_same_collection_child() {
    let mut conn = db::open(None).unwrap();
    let parent = tree("merge");
    fs::create_dir_all(parent.join("child")).unwrap();
    let child = parent.join("child");
    let audio = child.join("cortina.wav");
    write_wav(&audio, 1);
    let child_id = match add(&mut conn, &child, LibraryCollection::Effects).unwrap() {
        AddOutcome::Added { root_id, .. } => root_id,
        other => panic!("alta inesperada: {other:?}"),
    };
    indexer::sync_root(&mut conn, child_id, |_| {}).unwrap();
    let outcome = add(&mut conn, &parent, LibraryCollection::Effects).unwrap();
    let roots = list(&conn).unwrap();
    let results = search::search(&conn, "cortina", None, 10).unwrap();
    let folder_results = search::search(&conn, "child", None, 10).unwrap();

    assert_eq!(roots.len(), 1);
    assert_eq!(folder_results.len(), 1);
    let merged_path = Path::new(&results[0].path);
    assert!(merged_path.is_file());
    assert_eq!(
        merged_path.parent().and_then(Path::file_name),
        Some(std::ffi::OsStr::new("child"))
    );
    assert!(matches!(
        outcome,
        AddOutcome::Added { merged, .. } if merged == vec![child_id]
    ));
    let _ = fs::remove_dir_all(parent);
}

#[test]
fn nested_other_collection_remains_an_exception() {
    let mut conn = db::open(None).unwrap();
    let parent = tree("exception");
    fs::create_dir_all(parent.join("child")).unwrap();
    let child = parent.join("child");
    add(&mut conn, &parent, LibraryCollection::Music).unwrap();
    add(&mut conn, &child, LibraryCollection::Effects).unwrap();
    let roots = list(&conn).unwrap();
    let _ = fs::remove_dir_all(parent);

    assert_eq!(roots.len(), 2);
    assert_eq!(
        roots
            .iter()
            .filter(|root| root.collection == "effects")
            .count(),
        1
    );
}

#[test]
fn retiring_root_preserves_shared_track_edits_and_hides_catalog() {
    let mut conn = db::open(None).unwrap();
    let root = tree("remove");
    let audio = root.join("identificacion.wav");
    write_wav(&audio, 1);
    let root_id = match add(&mut conn, &root, LibraryCollection::Effects).unwrap() {
        AddOutcome::Added { root_id, .. } => root_id,
        other => panic!("alta inesperada: {other:?}"),
    };
    indexer::sync_root(&mut conn, root_id, |_| {}).unwrap();
    let key: String = conn
        .query_row(
            "SELECT path_key FROM library_track WHERE root_id=?1",
            params![root_id],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute("UPDATE track SET gain_db=4.0 WHERE path=?1", params![key])
        .unwrap();

    let retired = remove(&mut conn, root_id).unwrap();

    assert!(list(&conn).unwrap().is_empty());
    assert_eq!(retired.id, root_id);
    assert_eq!(root_retention::list(&conn).unwrap().len(), 1);
    assert!(search::search(&conn, "identificacion", None, 10)
        .unwrap()
        .is_empty());
    let gain: f64 = conn
        .query_row(
            "SELECT gain_db FROM track WHERE path=?1",
            params![key],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(gain, 4.0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn adding_the_same_retired_path_restores_without_duplicate() {
    let mut conn = db::open(None).unwrap();
    let root = tree("readd_retired");
    let root_id = match add(&mut conn, &root, LibraryCollection::Music).unwrap() {
        AddOutcome::Added { root_id, .. } => root_id,
        other => panic!("alta inesperada: {other:?}"),
    };
    remove(&mut conn, root_id).unwrap();

    let outcome = add(&mut conn, &root, LibraryCollection::Effects).unwrap();

    assert!(matches!(outcome, AddOutcome::Restored { root_id: id } if id == root_id));
    let roots = list(&conn).unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].collection, "effects");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn overlapping_a_retired_path_is_rejected_until_the_user_decides() {
    let mut conn = db::open(None).unwrap();
    let parent = tree("retired_overlap");
    let child = parent.join("child");
    fs::create_dir_all(&child).unwrap();
    let root_id = match add(&mut conn, &parent, LibraryCollection::Music).unwrap() {
        AddOutcome::Added { root_id, .. } => root_id,
        other => panic!("alta inesperada: {other:?}"),
    };
    remove(&mut conn, root_id).unwrap();

    assert_eq!(
        add(&mut conn, &child, LibraryCollection::Music).unwrap_err(),
        "library_root_overlaps_retired"
    );
    let _ = fs::remove_dir_all(parent);
}
