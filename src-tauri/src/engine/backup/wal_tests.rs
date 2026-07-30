use super::{create_package, inspect_package};
use crate::engine::persist::db;
use crate::model::AppConfig;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("lf-backup-wal-test-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn online_backup_includes_committed_wal_rows() {
    let dir = TestDir::new();
    let database = dir.0.join("tracks.db");
    let connection = db::open(Some(&database)).unwrap();
    connection
        .execute_batch("PRAGMA wal_autocheckpoint=0")
        .unwrap();
    for index in 0..250 {
        connection
            .execute(
                "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
             VALUES(?1,1,2,3,44100,2)",
                params![format!("C:/wal/{index}.mp3")],
            )
            .unwrap();
    }
    let package = dir.0.join("wal.lfbackup");

    let summary = create_package(&AppConfig::default(), &database, &package).unwrap();

    assert_eq!(summary.track_count, 250);
    assert!(database.with_extension("db-wal").exists());
}

#[test]
fn package_with_newer_database_schema_is_rejected() {
    let dir = TestDir::new();
    let database = dir.0.join("tracks.db");
    let connection = db::open(Some(&database)).unwrap();
    drop(connection);
    let package = dir.0.join("future.lfbackup");
    create_package(&AppConfig::default(), &database, &package).unwrap();
    Connection::open(&package)
        .unwrap()
        .execute("UPDATE lf_backup_manifest SET database_schema=999", [])
        .unwrap();

    assert_eq!(
        inspect_package(&package).unwrap_err(),
        "backup_database_newer"
    );
}
