//! Validación manual contra una Biblioteca real, siempre con salida explícita.
use super::restore::{prepare_restore_at, recover_pending_restore_at, take_restore_result_at};
use super::{create_package, inspect_package};
use crate::engine::persist::db;
use crate::model::AppConfig;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

#[test]
#[ignore = "requiere rutas LF_BACKUP_TEST_* y nunca modifica la base de origen"]
fn real_database_creates_and_restores_verified_package() {
    let source_db = required_path("LF_BACKUP_TEST_DB");
    let source_config = required_path("LF_BACKUP_TEST_CONFIG");
    let output = required_path("LF_BACKUP_TEST_OUTPUT");
    let restore_dir = required_path("LF_BACKUP_TEST_RESTORE_DIR");
    assert_safe_output(&output, &restore_dir, &source_db);
    assert!(!output.exists(), "la salida debe ser nueva");
    assert!(
        !restore_dir.exists(),
        "la carpeta de restauración debe ser nueva"
    );

    let config: AppConfig = serde_json::from_slice(&fs::read(&source_config).unwrap()).unwrap();
    let created = create_package(&config, &source_db, &output).unwrap();
    let inspected = inspect_package(&output).unwrap();
    println!(
        "respaldo verificado: perfiles={}, paletas={}, pistas={}, bytes={}",
        created.profile_count, created.palette_count, created.track_count, created.file_size
    );
    assert_eq!(created, inspected);
    assert!(created.track_count > 0);

    fs::create_dir_all(&restore_dir).unwrap();
    let current_db = restore_dir.join("tracks.db");
    drop(db::open(Some(&current_db)).unwrap());
    let current_config = AppConfig::default();
    fs::write(
        restore_dir.join("botonera_config.json"),
        serde_json::to_vec(&current_config).unwrap(),
    )
    .unwrap();
    prepare_restore_at(&output, &current_config, &restore_dir, &current_db).unwrap();
    recover_pending_restore_at(&restore_dir).unwrap();

    let restored_count: u64 = Connection::open(&current_db)
        .unwrap()
        .query_row("SELECT COUNT(*) FROM track", [], |row| row.get(0))
        .unwrap();
    assert_eq!(restored_count, created.track_count);
    assert!(
        take_restore_result_at(&restore_dir)
            .unwrap()
            .unwrap()
            .success
    );
}

fn required_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| panic!("falta {name}")))
}

fn assert_safe_output(output: &PathBuf, restore: &PathBuf, source: &PathBuf) {
    for path in [output, restore] {
        let text = path.to_string_lossy().to_ascii_lowercase();
        assert!(
            text.contains("lf-botonera-backup-validation"),
            "la salida debe incluir lf-botonera-backup-validation"
        );
    }
    assert_ne!(output, source, "nunca se escribe sobre tracks.db");
    assert_ne!(restore, source, "nunca se restaura sobre tracks.db");
}
