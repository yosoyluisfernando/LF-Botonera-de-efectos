/// Cache persistente de waveforms del editor.
use crate::engine::dsp::waveform::WaveEnvelope;
use crate::engine::persist::config_io;
use crate::engine::persist::db;
use crate::model::waveform_cache::WaveformCacheConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MB: u64 = 1024 * 1024;

#[derive(Default, Serialize)]
pub struct WaveformDiskStats {
    pub used_mb: f64,
    pub count: usize,
    pub max_mb: u32,
    pub max_age_days: u32,
}

#[derive(Default, Serialize, Deserialize)]
struct Index {
    items: HashMap<String, IndexEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
struct IndexEntry {
    file: String,
    bytes: u64,
    last_used: i64,
    mtime: i64,
    size: i64,
}

pub fn load(path: &str, mtime: i64, size: i64) -> Option<WaveEnvelope> {
    let key = key(path, mtime, size);
    let mut index = read_index();
    let entry = index.items.get_mut(&key)?;
    if entry.mtime != mtime || entry.size != size {
        return None;
    }
    let env = super::waveform_binary::read(&dir().join(&entry.file)).ok()?;
    entry.last_used = now();
    let _ = write_index(&index);
    Some(env)
}

pub fn save(path: &str, mtime: i64, size: i64, env: &WaveEnvelope) -> Result<(), String> {
    fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let key = key(path, mtime, size);
    let file = format!("{:016x}.wfc", fnv(&key));
    let bytes = super::waveform_binary::write(&dir().join(&file), env)?;
    let mut index = read_index();
    index.items.insert(
        key,
        IndexEntry {
            file,
            bytes,
            last_used: now(),
            mtime,
            size,
        },
    );
    write_index(&index)
}

pub fn cleanup(cfg: &WaveformCacheConfig) -> Result<WaveformDiskStats, String> {
    let cfg = cfg.sanitized();
    fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let mut index = read_index();
    let cutoff = now() - (cfg.max_age_days as i64 * 86_400);
    index.items.retain(|_, e| {
        let keep = e.last_used >= cutoff && dir().join(&e.file).exists();
        if !keep {
            let _ = fs::remove_file(dir().join(&e.file));
        }
        keep
    });
    evict_over_budget(&mut index, cfg.max_mb as u64 * MB);
    write_index(&index)?;
    Ok(stats_from(&index, &cfg))
}

pub fn stats(cfg: &WaveformCacheConfig) -> WaveformDiskStats {
    stats_from(&read_index(), &cfg.sanitized())
}

pub fn clear() -> Result<(), String> {
    if dir().exists() {
        fs::remove_dir_all(dir()).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(dir()).map_err(|e| e.to_string())
}

fn evict_over_budget(index: &mut Index, budget: u64) {
    while total_bytes(index) > budget {
        let Some(old_key) = index
            .items
            .iter()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(k, _)| k.clone())
        else {
            break;
        };
        if let Some(entry) = index.items.remove(&old_key) {
            let _ = fs::remove_file(dir().join(entry.file));
        }
    }
}

fn stats_from(index: &Index, cfg: &WaveformCacheConfig) -> WaveformDiskStats {
    WaveformDiskStats {
        used_mb: total_bytes(index) as f64 / MB as f64,
        count: index.items.len(),
        max_mb: cfg.max_mb,
        max_age_days: cfg.max_age_days,
    }
}

fn total_bytes(index: &Index) -> u64 {
    index.items.values().map(|e| e.bytes).sum()
}

fn read_index() -> Index {
    fs::read_to_string(index_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_index(index: &Index) -> Result<(), String> {
    fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let data = serde_json::to_string(index).map_err(|e| e.to_string())?;
    fs::write(index_path(), data).map_err(|e| e.to_string())
}

fn key(path: &str, mtime: i64, size: i64) -> String {
    format!("{}|{}|{}", db::normalize_key(path), mtime, size)
}

fn dir() -> PathBuf {
    config_io::get_data_dir().join("waveforms")
}

fn index_path() -> PathBuf {
    dir().join("index.json")
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(14695981039346656037u64, |h, b| {
        (h ^ b as u64).wrapping_mul(1099511628211)
    })
}

#[cfg(test)]
#[path = "waveform_disk_tests.rs"]
mod waveform_disk_tests;
