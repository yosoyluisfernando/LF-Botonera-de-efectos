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
fn v4_migration_adds_safe_library_retention() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA_V1).unwrap();
    conn.execute_batch(SCHEMA_V2).unwrap();
    conn.execute_batch(SCHEMA_V3).unwrap();
    conn.execute_batch(SCHEMA_V4).unwrap();
    conn.pragma_update(None, "user_version", 4).unwrap();
    conn.execute(
        "INSERT INTO library_root(path,path_key,collection,created_at)
         VALUES('D:/Music','d:/music','music',1)",
        [],
    )
    .unwrap();

    migrate(&conn).unwrap();

    let days: i64 = conn
        .query_row(
            "SELECT retention_days FROM library_setting WHERE id=1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let retired_at: Option<i64> = conn
        .query_row("SELECT retired_at FROM library_root", [], |row| row.get(0))
        .unwrap();
    assert_eq!(days, 30);
    assert_eq!(retired_at, None);
}

#[test]
fn v5_migration_adds_user_metadata_with_cascading_track_keys() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    for schema in [SCHEMA_V1, SCHEMA_V2, SCHEMA_V3, SCHEMA_V4, SCHEMA_V5] {
        conn.execute_batch(schema).unwrap();
    }
    conn.pragma_update(None, "user_version", 5).unwrap();
    conn.execute(
        "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
         VALUES('old.wav',1,2,3,44100,2)",
        [],
    )
    .unwrap();

    migrate(&conn).unwrap();
    conn.execute(
        "INSERT INTO track_user_metadata(path_key,title,updated_at)
         VALUES('old.wav','Interno',1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO track_keyword(path_key,keyword,keyword_key)
         VALUES('old.wav','Risa','risa')",
        [],
    )
    .unwrap();
    conn.execute("UPDATE track SET path='new.wav' WHERE path='old.wav'", [])
        .unwrap();

    let metadata: String = conn
        .query_row("SELECT path_key FROM track_user_metadata", [], |row| {
            row.get(0)
        })
        .unwrap();
    let keyword: String = conn
        .query_row("SELECT path_key FROM track_keyword", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        (metadata.as_str(), keyword.as_str()),
        ("new.wav", "new.wav")
    );
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
fn v3_migration_adds_browse_indexes() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA_V1).unwrap();
    conn.execute_batch(SCHEMA_V2).unwrap();
    conn.execute_batch(SCHEMA_V3).unwrap();
    conn.pragma_update(None, "user_version", 3).unwrap();
    migrate(&conn).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type='index' AND name LIKE 'library_track_browse_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn a_newer_schema_is_never_downgraded() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap();
    assert_eq!(migrate(&conn).unwrap_err(), "database_schema_newer");
}
