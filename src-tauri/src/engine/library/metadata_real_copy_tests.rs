//! Prueba manual ignorada: copia SQLite online y modifica solo la copia.
use super::service::LibraryService;
use crate::domain::library::metadata_fields::LibraryMetadataFields;
use crate::engine::backup;
use crate::engine::persist::db;
use crate::model::AppConfig;
use rusqlite::params;
use std::fs;
use std::path::PathBuf;

#[test]
#[ignore = "requiere LF_LIBRARY_REAL_DB; nunca modifica la base original"]
fn custom_metadata_survives_on_a_disposable_real_database_copy() {
    let source = PathBuf::from(std::env::var("LF_LIBRARY_REAL_DB").expect("falta base real"));
    assert!(source.is_file(), "la base indicada no existe");
    let directory = super::test_support::tree("metadata_real_copy");
    let copy = directory.join("library-copy.lfbackup");
    backup::create_package(&AppConfig::default(), &source, &copy).unwrap();

    let connection = db::open(Some(&copy)).unwrap();
    let (root, relative): (String, String) = connection
        .query_row(
            "SELECT lr.path,lt.relative_path
             FROM library_track lt JOIN library_root lr ON lr.id=lt.root_id
             WHERE lt.present=1 ORDER BY lt.path_key LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("la copia no contiene pistas activas");
    drop(connection);

    let path = PathBuf::from(root)
        .join(relative)
        .to_string_lossy()
        .to_string();
    let service = LibraryService::new(copy.clone());
    let tag = "prueba-segura-copia-real";
    let fields = LibraryMetadataFields {
        display_name: Some("Nombre temporal de prueba".into()),
        ..Default::default()
    };
    let saved = service
        .metadata_save(&[path.clone()], &fields, &[tag.into()], &[], false)
        .unwrap();
    assert_eq!(
        saved[0].display_name.as_deref(),
        Some("Nombre temporal de prueba")
    );
    assert!(saved[0].tags.iter().any(|value| value == tag));
    assert_eq!(service.search(tag, None, 10).unwrap().len(), 1);

    let connection = db::open(Some(&copy)).unwrap();
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM track_keyword WHERE keyword=?1",
            params![tag],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    drop(connection);
    fs::remove_dir_all(directory).unwrap();
}
