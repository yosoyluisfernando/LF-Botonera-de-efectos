use super::create_package;
use crate::engine::persist::db;
use crate::model::AppConfig;
use rusqlite::{params, Connection};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[test]
fn package_preserves_custom_metadata_and_keywords() {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let directory =
        std::env::temp_dir().join(format!("lf-backup-metadata-{}-{id}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let source = directory.join("tracks.db");
    let package = directory.join("metadata.lfbackup");
    let connection = db::open(Some(&source)).unwrap();
    connection
        .execute(
            "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
             VALUES(?1,1,2,3,44100,2)",
            params!["C:/audio/risa.mp3"],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO track_user_metadata(path_key,title,display_name,updated_at)
             VALUES(?1,?2,?3,1)",
            params!["C:/audio/risa.mp3", "Risa larga", "Risa con aplausos"],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO track_keyword(path_key,keyword,keyword_key,position)
             VALUES(?1,?2,?3,0)",
            params!["C:/audio/risa.mp3", "aplausos", "aplausos"],
        )
        .unwrap();
    drop(connection);

    create_package(&AppConfig::default(), &source, &package).unwrap();
    let restored = Connection::open(&package).unwrap();
    let title: String = restored
        .query_row(
            "SELECT title FROM track_user_metadata WHERE path_key=?1",
            params!["C:/audio/risa.mp3"],
            |row| row.get(0),
        )
        .unwrap();
    let keyword: String = restored
        .query_row(
            "SELECT keyword FROM track_keyword WHERE path_key=?1",
            params!["C:/audio/risa.mp3"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(title, "Risa larga");
    assert_eq!(keyword, "aplausos");
    drop(restored);
    fs::remove_dir_all(directory).unwrap();
}
