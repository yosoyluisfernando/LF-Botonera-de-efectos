//! Estado de conversión y porcentaje calculados exclusivamente en Rust.
use serde::Serialize;
use std::collections::HashMap;
#[cfg(not(target_os = "android"))]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
#[cfg(not(target_os = "android"))]
use tauri::Emitter;
#[derive(Clone, Debug, Serialize)]
pub struct ConversionSnapshot {
    pub revision: u64,
    pub path: String,
    pub status: String,
    pub percent: Option<u8>,
    pub error: Option<String>,
}
static JOBS: OnceLock<Mutex<HashMap<String, ConversionSnapshot>>> = OnceLock::new();
#[cfg(not(target_os = "android"))]
static REVISION: AtomicU64 = AtomicU64::new(0);
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
fn jobs() -> &'static Mutex<HashMap<String, ConversionSnapshot>> {
    JOBS.get_or_init(Default::default)
}
pub fn set_app(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
pub fn snapshots() -> Vec<ConversionSnapshot> {
    jobs().lock().unwrap().values().cloned().collect()
}
#[cfg(not(target_os = "android"))]
pub fn publish(path: &str, status: &str, percent: Option<u8>, error: Option<&str>) {
    let key = crate::engine::persist::db::normalize_key(path);
    let mut next = ConversionSnapshot {
        revision: 0,
        path: path.into(),
        status: status.into(),
        percent,
        error: error.map(str::to_string),
    };
    {
        let mut jobs = jobs().lock().unwrap();
        if jobs.get(&key).is_some_and(|old| {
            old.status == next.status && old.percent == next.percent && old.error == next.error
        }) {
            return;
        }
        if status == "converting"
            && jobs
                .get(&key)
                .is_some_and(|j| matches!(j.status.as_str(), "ready" | "failed" | "native"))
        {
            return;
        }
        next.revision = REVISION.fetch_add(1, Ordering::Relaxed) + 1;
        // Retener los activos; los finalizados más antiguos no crecen indefinidamente.
        if jobs.len() >= 1024 {
            jobs.retain(|_, j| !matches!(j.status.as_str(), "ready" | "failed" | "native"));
        }
        jobs.insert(key, next.clone());
    }
    if let Some(app) = APP.get() {
        let _ = app.emit("audio-convert-status", next);
    }
}
#[cfg(not(target_os = "android"))]
pub fn advance(path: &str, line: &str, duration_s: f64) {
    if let Some(value) = line
        .strip_prefix("out_time_us=")
        .and_then(|s| s.parse::<u64>().ok())
    {
        publish(path, "converting", percent(value, duration_s), None);
    }
}
#[cfg(not(target_os = "android"))]
fn percent(micros: u64, duration_s: f64) -> Option<u8> {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return None;
    }
    Some(((micros as f64 / 1_000_000.0 / duration_s * 100.0).floor() as u64).min(99) as u8)
}
#[cfg(all(test, not(target_os = "android")))]
mod tests {
    use super::*;
    #[test]
    fn percentage_comes_from_encoded_time_and_waits_for_validation() {
        assert_eq!(percent(5_000_000, 20.0), Some(25));
        assert_eq!(percent(20_000_000, 20.0), Some(99));
        assert_eq!(percent(40_000_000, 20.0), Some(99));
        assert_eq!(percent(0, 0.0), None);
        assert_eq!(percent(10, f64::NAN), None);
    }
}
