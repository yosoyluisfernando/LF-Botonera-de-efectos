use super::*;
use crate::domain::library::LibraryCollection;
use crate::engine::library::search;
use crate::engine::library::test_support::{add_root, tree, write_wav};
use crate::engine::persist::db;
use std::fs;

#[test]
fn create_modify_and_delete_touch_only_the_changed_file() {
    let root = tree("incremental_file");
    let audio = root.join("Nueva Sirena.wav");
    let mut conn = db::open(None).unwrap();
    add_root(&mut conn, &root, LibraryCollection::Effects);
    write_wav(&audio, 1);

    let created = apply_paths(&mut conn, std::slice::from_ref(&audio)).unwrap();
    let repeated_event = apply_paths(&mut conn, std::slice::from_ref(&audio)).unwrap();
    write_wav(&audio, 2);
    let modified = apply_paths(&mut conn, std::slice::from_ref(&audio)).unwrap();
    fs::remove_file(&audio).unwrap();
    let removed = apply_paths(&mut conn, std::slice::from_ref(&audio)).unwrap();

    assert_eq!(created.updated, 1);
    assert_eq!(repeated_event.updated, 1);
    assert_eq!(modified.updated, 1);
    assert_eq!(removed.removed, 1);
    assert!(search::search(&conn, "sirena", None, 10)
        .unwrap()
        .is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn directory_event_requests_only_its_owner_reconciliation() {
    let root = tree("incremental_dir");
    let child = root.join("nueva");
    fs::create_dir_all(&child).unwrap();
    let mut conn = db::open(None).unwrap();
    let root_id = add_root(&mut conn, &root, LibraryCollection::Music);

    let report = apply_paths(&mut conn, &[child]).unwrap();

    assert_eq!(report.reconcile_roots, [root_id]);
    let _ = fs::remove_dir_all(root);
}
