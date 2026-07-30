//! Prueba manual destructiva solo sobre una copia explícita de tracks.db.
use super::{root_purge, root_retention, root_store, search};
use crate::engine::persist::db;
use rusqlite::params;
use std::collections::HashSet;
use std::path::PathBuf;

#[test]
#[ignore = "requiere LF_RETENTION_TEST_COPY y modifica exclusivamente esa copia"]
fn real_database_copy_completes_retire_restore_and_expiry() {
    let path = PathBuf::from(
        std::env::var("LF_RETENTION_TEST_COPY").expect("falta LF_RETENTION_TEST_COPY"),
    );
    assert!(
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains("retention-work")),
        "la copia debe incluir retention-work en su nombre"
    );
    assert_ne!(
        path.canonicalize().unwrap(),
        db::db_path().canonicalize().unwrap(),
        "nunca se prueba sobre tracks.db real"
    );
    let mut connection = db::open(Some(&path)).unwrap();
    let root_id: i64 = connection
        .query_row(
            "SELECT root_id FROM library_track lt
             JOIN library_root lr ON lr.id=lt.root_id
             WHERE lr.enabled=1 GROUP BY root_id HAVING COUNT(*)>=2
             ORDER BY COUNT(*) DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let paths = {
        let mut statement = connection
            .prepare("SELECT path_key FROM library_track WHERE root_id=?1 LIMIT 2")
            .unwrap();
        statement
            .query_map(params![root_id], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };
    let protected = HashSet::from([paths[0].clone()]);
    let active_before = search::search(&connection, "", None, 10).unwrap().len();

    let retired = root_retention::retire_at(&connection, root_id, 10_000).unwrap();
    assert!(retired.purge_after > 10_000);
    assert!(root_store::list(&connection)
        .unwrap()
        .iter()
        .all(|root| root.id != root_id));
    assert!(search::search(&connection, "", None, 10).unwrap().len() <= active_before);

    root_store::restore(&mut connection, root_id).unwrap();
    assert!(root_store::list(&connection)
        .unwrap()
        .iter()
        .any(|root| root.id == root_id));

    root_retention::retire_at(&connection, root_id, 20_000).unwrap();
    connection
        .execute(
            "UPDATE library_root SET purge_after=20_001 WHERE id=?1",
            params![root_id],
        )
        .unwrap();
    let report = root_purge::purge_expired_at(&mut connection, &protected, 20_001).unwrap();

    assert_eq!(report.roots, 1);
    assert!(report.catalog_tracks >= 2);
    assert_eq!(report.protected_track_meta, 1);
    assert!(connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM track WHERE path=?1)",
            params![&paths[0]],
            |row| row.get::<_, bool>(0),
        )
        .unwrap());
    assert!(!connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM track WHERE path=?1)",
            params![&paths[1]],
            |row| row.get::<_, bool>(0),
        )
        .unwrap());
}
