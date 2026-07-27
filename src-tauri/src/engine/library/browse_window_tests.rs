use super::*;
use crate::domain::library::LibraryCollection;
use crate::engine::library::test_support::{add_root, tree};
use crate::engine::persist::db;
use rusqlite::params;
use std::fs;

#[test]
fn absolute_window_reports_total_and_requested_position() {
    let root = tree("browse-window");
    let mut connection = db::open(None).unwrap();
    let root_id = add_root(&mut connection, &root, LibraryCollection::Music);
    let transaction = connection.transaction().unwrap();
    for index in 0..400 {
        let key = format!("test:/window_{index:04}.mp3");
        let name = format!("Pista {index:04}");
        transaction
            .execute(
                "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
                 VALUES(?1,1,1,180,44100,2)",
                params![key],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO library_track(path_key,root_id,collection,relative_path,
                 file_name,extension,metadata_state,last_seen_at)
                 VALUES(?1,?2,'music',?3,?3,'mp3','complete',0)",
                params![key, root_id, name],
            )
            .unwrap();
    }
    transaction.commit().unwrap();
    let page = browse_window(&connection, Some("music"), Some(root_id), None, 175, 80).unwrap();
    assert_eq!(page.total, 400);
    assert_eq!(page.offset, 175);
    assert_eq!(page.items.len(), 80);
    assert_eq!(page.items[0].file_name, "Pista 0175");
    drop(connection);
    let _ = fs::remove_dir_all(root);
}
