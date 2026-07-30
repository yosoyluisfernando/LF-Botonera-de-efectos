use super::embedded_tags;
use crate::domain::library::metadata_fields::LibraryMetadataFields;
use crate::engine::library::metadata;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

#[test]
fn verified_tag_write_can_commit_without_changing_audio_properties() {
    let dir = test_dir();
    let audio = dir.join("tono.wav");
    write_wav(&audio);
    let before = metadata::read(&audio.to_string_lossy()).unwrap();
    let fields = LibraryMetadataFields {
        title: Some("Tono seguro".into()),
        artist: Some("LF".into()),
        ..Default::default()
    };

    let pending = embedded_tags::prepare_at(&dir, &audio.to_string_lossy(), &fields)
        .unwrap()
        .unwrap();
    pending.install().unwrap();
    pending.commit().unwrap();

    let after = metadata::read(&audio.to_string_lossy()).unwrap();
    assert_eq!(after.title.as_deref(), Some("Tono seguro"));
    assert_eq!(after.artist.as_deref(), Some("LF"));
    assert_eq!(after.duration_s, before.duration_s);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn rollback_restores_original_bytes() {
    let dir = test_dir();
    let audio = dir.join("original.wav");
    write_wav(&audio);
    let original = fs::read(&audio).unwrap();
    let fields = LibraryMetadataFields {
        title: Some("Temporal".into()),
        ..Default::default()
    };

    let pending = embedded_tags::prepare_at(&dir, &audio.to_string_lossy(), &fields)
        .unwrap()
        .unwrap();
    pending.install().unwrap();
    pending.rollback().unwrap();

    assert_eq!(fs::read(&audio).unwrap(), original);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn startup_recovery_rolls_back_an_uncommitted_install() {
    let dir = test_dir();
    let audio = dir.join("interrumpido.wav");
    write_wav(&audio);
    let original = fs::read(&audio).unwrap();
    let fields = LibraryMetadataFields {
        title: Some("No confirmado".into()),
        ..Default::default()
    };
    let pending = embedded_tags::prepare_at(&dir, &audio.to_string_lossy(), &fields)
        .unwrap()
        .unwrap();
    pending.install().unwrap();

    super::tag_write_journal::recover_at(&dir).unwrap();

    assert_eq!(fs::read(&audio).unwrap(), original);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn startup_recovery_finishes_a_committed_install() {
    let dir = test_dir();
    let audio = dir.join("confirmado.wav");
    write_wav(&audio);
    let fields = LibraryMetadataFields {
        title: Some("Confirmado".into()),
        ..Default::default()
    };
    let pending = embedded_tags::prepare_at(&dir, &audio.to_string_lossy(), &fields)
        .unwrap()
        .unwrap();
    pending.install().unwrap();
    pending.mark_committed().unwrap();

    super::tag_write_journal::recover_at(&dir).unwrap();

    let result = metadata::read(&audio.to_string_lossy()).unwrap();
    assert_eq!(result.title.as_deref(), Some("Confirmado"));
    let _ = fs::remove_dir_all(dir);
}

fn test_dir() -> PathBuf {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("lf-embedded-tags-{}-{id}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_wav(path: &Path) {
    let samples = [0i16; 800];
    let data_size = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&16000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).unwrap();
}
