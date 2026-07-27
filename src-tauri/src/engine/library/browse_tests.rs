use super::*;
use crate::domain::library::LibraryCollection;
use crate::engine::library::test_support::{add_root, tree};
use crate::engine::persist::db;
use rusqlite::params;
use std::collections::HashSet;
use std::fs;

#[test]
fn cursor_pages_do_not_repeat_or_skip_catalog_rows() {
    let (connection, root) = catalog(235);
    let mut cursor = None;
    let mut paths = Vec::new();
    let mut page_sizes = Vec::new();
    loop {
        let page = browse(
            &connection,
            None,
            100,
            cursor.as_ref(),
            BrowseDirection::Forward,
        )
        .unwrap();
        page_sizes.push(page.items.len());
        paths.extend(page.items.into_iter().map(|item| item.path));
        cursor = page.next_cursor;
        if !page.has_more {
            break;
        }
    }
    let unique = paths.iter().collect::<HashSet<_>>();
    assert_eq!(page_sizes, [100, 100, 35]);
    assert_eq!(paths.len(), 235);
    assert_eq!(unique.len(), 235);
    drop(connection);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn collection_filter_and_cursor_survive_catalog_changes() {
    let (connection, root) = catalog(12);
    connection
        .execute(
            "UPDATE library_track SET collection='effects' WHERE file_name<'Pista 000004'",
            [],
        )
        .unwrap();
    let first = browse(
        &connection,
        Some("music"),
        4,
        None,
        BrowseDirection::Forward,
    )
    .unwrap();
    assert!(first.items.iter().all(|item| item.collection == "music"));
    connection
        .execute(
            "UPDATE library_track SET present=0 WHERE file_name='Pista 000009'",
            [],
        )
        .unwrap();
    let second = browse(
        &connection,
        Some("music"),
        20,
        first.next_cursor.as_ref(),
        BrowseDirection::Forward,
    )
    .unwrap();
    let first_paths = first
        .items
        .iter()
        .map(|item| &item.path)
        .collect::<HashSet<_>>();
    assert!(second
        .items
        .iter()
        .all(|item| !first_paths.contains(&item.path)));
    assert!(second
        .items
        .iter()
        .all(|item| item.file_name != "Pista 000009"));
    drop(connection);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn backward_cursor_supports_lazy_scrolling_up() {
    let (connection, root) = catalog(235);
    let mut cursor = None;
    let mut sizes = Vec::new();
    loop {
        let page = browse(
            &connection,
            None,
            100,
            cursor.as_ref(),
            BrowseDirection::Backward,
        )
        .unwrap();
        sizes.push(page.items.len());
        cursor = page.previous_cursor;
        if !page.has_more {
            break;
        }
    }
    assert_eq!(sizes, [100, 100, 35]);
    drop(connection);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn root_and_folder_scope_only_returns_descendants() {
    let (connection, root) = catalog(12);
    let root_id = connection
        .query_row("SELECT id FROM library_root LIMIT 1", [], |row| row.get(0))
        .unwrap();
    connection
        .execute(
            "UPDATE library_track SET relative_path='Rock/' || file_name
             WHERE file_name<'Pista 000006'",
            [],
        )
        .unwrap();
    let page = browse_scoped(
        &connection,
        Some("music"),
        Some(root_id),
        Some("Rock"),
        20,
        None,
        BrowseDirection::Forward,
    )
    .unwrap();
    assert_eq!(page.items.len(), 6);
    assert!(page.items.iter().all(|item| item.path.contains("Rock")));
    drop(connection);
    let _ = fs::remove_dir_all(root);
}

fn catalog(count: usize) -> (rusqlite::Connection, std::path::PathBuf) {
    let root = tree("browse");
    let mut connection = db::open(None).unwrap();
    let root_id = add_root(&mut connection, &root, LibraryCollection::Music);
    let transaction = connection.transaction().unwrap();
    for index in 0..count {
        let key = format!("test:/pista_{index:06}.mp3");
        let name = format!("Pista {index:06}");
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
        transaction
            .execute(
                "INSERT INTO library_track_search(path_key,search_text) VALUES(?1,?2)",
                params![key, name],
            )
            .unwrap();
    }
    transaction.commit().unwrap();
    (connection, root)
}
