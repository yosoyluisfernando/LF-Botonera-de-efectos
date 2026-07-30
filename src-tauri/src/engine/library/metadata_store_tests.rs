use super::*;
use crate::domain::library::metadata_fields::LibraryMetadataFields;
use crate::domain::library::root_plan::LibraryCollection;
use crate::engine::library::test_support::{add_root, tree, write_wav};
use crate::engine::persist::db;
use std::fs;

fn catalog() -> (rusqlite::Connection, std::path::PathBuf, String, i64) {
    let root = tree("user_metadata");
    let audio = root.join("Risa divertida.wav");
    write_wav(&audio, 1);
    let mut connection = db::open(None).unwrap();
    let root_id = add_root(&mut connection, &root, LibraryCollection::Effects);
    indexer::sync_root(&mut connection, root_id, |_| {}).unwrap();
    let path = search::search(&connection, "", None, 1).unwrap()[0]
        .path
        .clone();
    (connection, root, path, root_id)
}

#[test]
fn overrides_and_normalized_tags_feed_get_search_and_browse() {
    let (mut connection, root, path, _) = catalog();
    let fields = LibraryMetadataFields {
        title: Some("Carcajada breve".into()),
        album_artist: Some("Colección LF".into()),
        display_name: Some("Risa con aplausos".into()),
        category: Some("Risas".into()),
        ..Default::default()
    };
    metadata_store::save(
        &mut connection,
        std::slice::from_ref(&path),
        &fields,
        &[
            "Cómica".into(),
            "comica".into(),
            "Aplausos".into(),
            "Boom".into(),
        ],
        &[],
    )
    .unwrap();

    let item = metadata_query::items(&connection, std::slice::from_ref(&path))
        .unwrap()
        .remove(0);
    assert_eq!(item.title.as_deref(), Some("Carcajada breve"));
    assert_eq!(item.album_artist.as_deref(), Some("Colección LF"));
    assert_eq!(item.tags, ["Cómica", "Aplausos", "Boom"]);
    assert_eq!(
        search::search(&connection, "comica", None, 10)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        search::search(&connection, "boom", None, 10).unwrap().len(),
        1
    );
    let page = browse::browse(
        &connection,
        None,
        10,
        None,
        browse::BrowseDirection::Forward,
    )
    .unwrap();
    assert_eq!(page.items[0].title.as_deref(), Some("Carcajada breve"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn batch_is_atomic_when_any_path_is_missing() {
    let (mut connection, root, path, _) = catalog();
    let fields = LibraryMetadataFields {
        genre: Some("Comedia".into()),
        ..Default::default()
    };
    let error = metadata_store::save(
        &mut connection,
        &[
            path.clone(),
            root.join("ausente.wav").to_string_lossy().to_string(),
        ],
        &fields,
        &["Risa".into()],
        &[],
    )
    .unwrap_err();

    assert_eq!(error, "library_track_not_found");
    let item = metadata_query::items(&connection, &[path])
        .unwrap()
        .remove(0);
    assert_eq!(item.genre, None);
    assert!(item.tags.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn edits_survive_reindex_and_removal_uses_normalized_tag() {
    let (mut connection, root, path, root_id) = catalog();
    metadata_store::save(
        &mut connection,
        std::slice::from_ref(&path),
        &LibraryMetadataFields {
            description: Some("Risa de público".into()),
            ..Default::default()
        },
        &["Público".into()],
        &[],
    )
    .unwrap();
    write_wav(std::path::Path::new(&path), 2);
    indexer::sync_root(&mut connection, root_id, |_| {}).unwrap();
    metadata_store::save(
        &mut connection,
        std::slice::from_ref(&path),
        &LibraryMetadataFields::default(),
        &[],
        &["publico".into()],
    )
    .unwrap();

    let item = metadata_query::items(&connection, &[path])
        .unwrap()
        .remove(0);
    assert_eq!(item.description.as_deref(), Some("Risa de público"));
    assert!(item.tags.is_empty());
    assert_eq!(
        metadata_query::suggestions(&connection, "pub", 10).unwrap(),
        Vec::<String>::new()
    );
    let _ = fs::remove_dir_all(root);
}
