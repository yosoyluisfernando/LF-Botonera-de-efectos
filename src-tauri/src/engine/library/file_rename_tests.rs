use super::*;
use crate::domain::button::defaults::new_button;
use rusqlite::params;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("lf-library-rename-{}-{id}", std::process::id()));
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
fn rename_preserves_database_metadata_and_all_config_references() {
    let dir = TestDir::new();
    let (database, old, new) = fixture(&dir.0);
    let mut config = config_with_every_reference(&old);

    let result = rename_at(&database, &dir.0, &mut config, &old, "nueva.mp3").unwrap();

    assert_eq!(result.1, new);
    assert!(!Path::new(&old).exists());
    assert_eq!(fs::read(&new).unwrap(), b"audio");
    let connection = db::open(Some(&database)).unwrap();
    let new_key = db::normalize_key(&new);
    assert_eq!(
        connection
            .query_row(
                "SELECT cue_start_s FROM track WHERE path=?1",
                params![new_key],
                |row| row.get::<_, f64>(0)
            )
            .unwrap(),
        1.25
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT keyword FROM track_keyword WHERE path_key=?1",
                params![new_key],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "Risa"
    );
    assert_eq!(path_rewrite::replace_in_config(&mut config, &old, &new), 0);
    let saved: AppConfig =
        serde_json::from_slice(&fs::read(dir.0.join("botonera_config.json")).unwrap()).unwrap();
    assert_eq!(saved.player.tracks[0].path, new);
}

#[test]
fn recovery_completes_interruption_after_physical_rename() {
    let dir = TestDir::new();
    let (database, old, new) = fixture(&dir.0);
    let old_config = config_with_every_reference(&old);
    let mut new_config = old_config.clone();
    path_rewrite::replace_in_config(&mut new_config, &old, &new);
    config_io::save_config_at(&dir.0.join("botonera_config.json"), &old_config).unwrap();
    rename_journal::store_at(
        &dir.0,
        &RenameJournal {
            old_path: old.clone(),
            new_path: new.clone(),
            old_config,
            new_config,
        },
    )
    .unwrap();
    fs::rename(&old, &new).unwrap();

    recover_pending_at(&database, &dir.0).unwrap();

    let connection = db::open(Some(&database)).unwrap();
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM track WHERE path=?1)",
            params![db::normalize_key(&new)],
            |row| row.get(0),
        )
        .unwrap();
    assert!(exists);
    let saved: AppConfig =
        serde_json::from_slice(&fs::read(dir.0.join("botonera_config.json")).unwrap()).unwrap();
    assert_eq!(saved.player.tracks[0].path, new);
    assert!(rename_journal::load_at(&dir.0).unwrap().is_none());
}

fn fixture(dir: &Path) -> (PathBuf, String, String) {
    let old_path = dir.join("vieja.mp3");
    let new_path = dir.join("nueva.mp3");
    fs::write(&old_path, b"audio").unwrap();
    let database = dir.join("tracks.db");
    let connection = db::open(Some(&database)).unwrap();
    let root = dir.to_string_lossy().to_string();
    let old = old_path.to_string_lossy().to_string();
    let key = db::normalize_key(&old);
    connection
        .execute(
            "INSERT INTO library_root(path,path_key,collection,created_at,state)
             VALUES(?1,?2,'effects',1,'ready')",
            params![root, db::normalize_key(&dir.to_string_lossy())],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO track(path,mtime,size,duration_s,sample_rate,channels,cue_start_s)
             VALUES(?1,1,5,3,44100,2,1.25)",
            params![key],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO library_track(path_key,root_id,collection,relative_path,file_name,
             extension,metadata_state,last_seen_at)
             VALUES(?1,1,'effects','vieja.mp3','vieja.mp3','mp3','complete',1)",
            params![key],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO track_keyword(path_key,keyword,keyword_key) VALUES(?1,'Risa','risa')",
            params![key],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO library_track_search(path_key,search_text) VALUES(?1,'vieja risa')",
            params![key],
        )
        .unwrap();
    (database, old, new_path.to_string_lossy().to_string())
}

fn config_with_every_reference(path: &str) -> AppConfig {
    let mut config = AppConfig::default();
    let mut button = new_button("audio", 0, "", "", "");
    button.path = path.into();
    config.profiles[0].paletas[0].botones.push(button.clone());
    config.profiles[0].fixed_buttons.push(button.clone());
    config.fixed_panel.global_buttons.push(button.clone());
    config.player.tracks.push(button);
    config
}
