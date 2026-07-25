use super::super::service::LibraryService;
use crate::engine::library::test_support::{tree, write_wav};
use std::fs;
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[ignore = "usa el observador real del sistema de archivos"]
fn watcher_indexes_modifies_and_removes_a_real_file() {
    let audio_root = tree("watch_audio");
    let database_root = tree("watch_db");
    let database = database_root.join("watch.sqlite");
    let audio = audio_root.join("Alerta Nueva.wav");
    let existing = audio_root.join("Cortina Previa.wav");
    let service = LibraryService::new(database);
    service
        .add_root(&audio_root.to_string_lossy(), "effects")
        .unwrap();
    write_wav(&existing, 1);
    service.start_monitoring().unwrap();
    assert!(service.status().unwrap().monitoring);
    assert!(wait_until(|| {
        service
            .search("cortina previa", None, 10)
            .map(|rows| rows.len() == 1)
            .unwrap_or(false)
    }));

    write_wav(&audio, 1);
    assert!(wait_until(|| {
        service
            .search("alerta nueva", None, 10)
            .map(|rows| rows.len() == 1)
            .unwrap_or(false)
    }));

    write_wav(&audio, 2);
    assert!(wait_until(|| {
        service
            .search("alerta nueva", None, 10)
            .ok()
            .and_then(|rows| rows.first().map(|row| row.duration_s >= 2.0))
            .unwrap_or(false)
    }));

    fs::remove_file(&audio).unwrap();
    assert!(wait_until(|| {
        service
            .search("alerta nueva", None, 10)
            .map(|rows| rows.is_empty())
            .unwrap_or(false)
    }));
    drop(service);
    fs::remove_dir_all(audio_root).unwrap();
    fs::remove_dir_all(database_root).unwrap();
}

fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}
