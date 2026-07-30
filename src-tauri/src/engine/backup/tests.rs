use super::package::{create_package, inspect_package, inspect_package_data};
use super::restore::{prepare_restore_at, recover_pending_restore_at, take_restore_result_at};
use crate::engine::persist::db;
use crate::model::AppConfig;
use rusqlite::{params, Connection};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("lf-backup-test-{}-{id}", std::process::id()));
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
fn package_round_trip_preserves_config_and_database() {
    let dir = TestDir::new();
    let source = dir.0.join("tracks.db");
    make_database(&source, 3);
    let config = config_named("Respaldo ágil");
    let package = dir.0.join("válido.lfbackup");

    let created = create_package(&config, &source, &package).unwrap();
    let inspected = inspect_package_data(&package).unwrap();

    assert_eq!(created.track_count, 3);
    assert_eq!(created.profile_count, config.profiles.len() as u64);
    assert!(created.integrity_ok);
    assert!(!created.audio_included);
    let restored: AppConfig = serde_json::from_str(&inspected.config_json).unwrap();
    assert_eq!(restored.profiles[0].name, "Respaldo ágil");
}

#[test]
fn existing_destination_is_never_overwritten() {
    let dir = TestDir::new();
    let source = dir.0.join("tracks.db");
    make_database(&source, 1);
    let package = dir.0.join("existente.lfbackup");
    fs::write(&package, b"conservar").unwrap();

    let error = create_package(&AppConfig::default(), &source, &package).unwrap_err();

    assert_eq!(error, "backup_destination_exists");
    assert_eq!(fs::read(&package).unwrap(), b"conservar");
}

#[test]
fn truncated_and_inconsistent_packages_are_rejected() {
    let dir = TestDir::new();
    let source = dir.0.join("tracks.db");
    make_database(&source, 2);
    let package = dir.0.join("original.lfbackup");
    create_package(&AppConfig::default(), &source, &package).unwrap();
    let inconsistent = dir.0.join("inconsistente.lfbackup");
    fs::copy(&package, &inconsistent).unwrap();
    Connection::open(&inconsistent)
        .unwrap()
        .execute(
            "UPDATE lf_backup_manifest SET track_count=track_count+1",
            [],
        )
        .unwrap();
    assert_eq!(
        inspect_package(&inconsistent).unwrap_err(),
        "backup_manifest_mismatch"
    );

    let truncated = dir.0.join("truncado.lfbackup");
    let bytes = fs::read(&package).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&truncated)
        .unwrap();
    file.write_all(&bytes[..bytes.len() / 2]).unwrap();
    drop(file);
    assert!(inspect_package(&truncated).is_err());
}

#[test]
fn restore_is_atomic_and_leaves_an_emergency_backup() {
    let dir = TestDir::new();
    let current_db = dir.0.join("tracks.db");
    make_database(&current_db, 1);
    let current = config_named("Actual");
    write_config(&dir.0, &current);
    let selected = desired_package(&dir.0, 4, "Restaurado");

    let prepared = prepare_restore_at(&selected, &current, &dir.0, &current_db).unwrap();
    recover_pending_restore_at(&dir.0).unwrap();

    assert_eq!(inspect_package(&current_db).unwrap().track_count, 4);
    assert_eq!(read_config(&dir.0).profiles[0].name, "Restaurado");
    assert!(Path::new(&prepared.emergency_backup_path).exists());
    let result = take_restore_result_at(&dir.0).unwrap().unwrap();
    assert!(result.success);
    assert_eq!(
        result.summary.unwrap().backup_id,
        prepared.summary.backup_id
    );
}

#[test]
fn interrupted_swap_is_completed_on_next_start() {
    let dir = TestDir::new();
    let current_db = dir.0.join("tracks.db");
    make_database(&current_db, 1);
    let current = config_named("Actual");
    write_config(&dir.0, &current);
    let selected = desired_package(&dir.0, 5, "Tras interrupción");
    prepare_restore_at(&selected, &current, &dir.0, &current_db).unwrap();
    fs::rename(&current_db, dir.0.join("tracks.db.restore-old")).unwrap();

    recover_pending_restore_at(&dir.0).unwrap();

    assert_eq!(inspect_package(&current_db).unwrap().track_count, 5);
    assert_eq!(read_config(&dir.0).profiles[0].name, "Tras interrupción");
}

#[test]
fn invalid_staging_rolls_back_without_losing_current_data() {
    let dir = TestDir::new();
    let current_db = dir.0.join("tracks.db");
    make_database(&current_db, 2);
    let current = config_named("Seguro");
    write_config(&dir.0, &current);
    let selected = desired_package(&dir.0, 6, "No debe quedar");
    prepare_restore_at(&selected, &current, &dir.0, &current_db).unwrap();
    fs::write(
        dir.0.join(".restore-transaction").join("staged.lfbackup"),
        b"corrupto",
    )
    .unwrap();

    recover_pending_restore_at(&dir.0).unwrap();

    assert_eq!(inspect_package(&current_db).unwrap().track_count, 2);
    assert_eq!(read_config(&dir.0).profiles[0].name, "Seguro");
    assert!(!take_restore_result_at(&dir.0).unwrap().unwrap().success);
}

fn desired_package(dir: &Path, tracks: usize, name: &str) -> PathBuf {
    let db_path = dir.join(format!("desired-{tracks}.db"));
    make_database(&db_path, tracks);
    let path = dir.join(format!("desired-{tracks}.lfbackup"));
    create_package(&config_named(name), &db_path, &path).unwrap();
    path
}

fn make_database(path: &Path, tracks: usize) {
    let connection = db::open(Some(path)).unwrap();
    for index in 0..tracks {
        connection
            .execute(
                "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels)
             VALUES(?1,1,2,3,44100,2)",
                params![format!("C:/audio/{index}.mp3")],
            )
            .unwrap();
    }
}

fn config_named(name: &str) -> AppConfig {
    let mut config = AppConfig::default();
    config.profiles[0].name = name.into();
    config
}

fn write_config(dir: &Path, config: &AppConfig) {
    fs::write(
        dir.join("botonera_config.json"),
        serde_json::to_string(config).unwrap(),
    )
    .unwrap();
}

fn read_config(dir: &Path) -> AppConfig {
    serde_json::from_slice(&fs::read(dir.join("botonera_config.json")).unwrap()).unwrap()
}
