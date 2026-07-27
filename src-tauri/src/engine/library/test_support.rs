use super::root_store::{self, AddOutcome};
use crate::domain::library::root_plan::LibraryCollection;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TREE_ID: AtomicU64 = AtomicU64::new(1);

pub fn tree(name: &str) -> PathBuf {
    let id = NEXT_TREE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("lf_library_{name}_{}_{id}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

pub fn write_wav(path: &Path, seconds: u32) {
    let rate = 8_000u32;
    let channels = 1u16;
    let bits = 16u16;
    let data_size = rate * seconds * channels as u32 * (bits as u32 / 8);
    let mut bytes = Vec::with_capacity(44 + data_size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&bits.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    bytes.resize(44 + data_size as usize, 0);
    fs::write(path, bytes).unwrap();
}

pub fn add_root(conn: &mut Connection, path: &Path, collection: LibraryCollection) -> i64 {
    match root_store::add(conn, path, collection).unwrap() {
        AddOutcome::Added { root_id, .. } => root_id,
        other => panic!("alta inesperada: {other:?}"),
    }
}
