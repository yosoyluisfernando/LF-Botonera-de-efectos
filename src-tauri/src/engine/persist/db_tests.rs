use super::*;

#[test]
fn migrate_sets_user_version_and_tables() {
    let conn = open(None).unwrap();
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
    for table in ["track", "library_root", "library_track"] {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        assert_eq!(conn.query_row(&sql, [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    }
}

#[test]
fn migrate_is_idempotent() {
    let conn = open(None).unwrap();
    migrate(&conn).unwrap();
    migrate(&conn).unwrap();
}

#[test]
fn normalize_key_is_consistent() {
    let a = normalize_key("C:/Audio/Risa.mp3");
    assert_eq!(a, normalize_key(&a));
}

#[test]
fn v1_migration_preserves_tracks_and_adds_roots() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA_V1).unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
    conn.execute(
        "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
         VALUES('c:/audio/a.mp3',1,2,3,44100,2)",
        [],
    )
    .unwrap();
    migrate(&conn).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM track", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn v2_migration_adds_catalog_without_changing_roots() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA_V1).unwrap();
    conn.execute_batch(SCHEMA_V2).unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
    conn.execute(
        "INSERT INTO library_root(path,path_key,collection,created_at)
         VALUES('D:/Music','d:/music','music',1)",
        [],
    )
    .unwrap();
    migrate(&conn).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM library_root", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM library_track", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn a_newer_schema_is_never_downgraded() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap();
    assert_eq!(migrate(&conn).unwrap_err(), "database_schema_newer");
}
