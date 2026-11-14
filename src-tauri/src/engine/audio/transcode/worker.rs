//! Cola única de conversión, con estados publicados antes de encolar.
use super::{cache, ffmpeg, probe};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Sender},
    Arc, OnceLock,
};
use std::time::Duration;
use tauri::Emitter;

use super::state::{State, Status};
struct Shared {
    state: Arc<State>,
    sender: Sender<(String, PathBuf)>,
}
static SHARED: OnceLock<Shared> = OnceLock::new();
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
#[derive(Clone, Serialize)]
struct Event<'a> {
    path: &'a str,
    status: &'a str,
    error: Option<&'a str>,
}
pub fn set_app(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
fn emit(path: &str, status: &str, error: Option<&str>) {
    if let Some(app) = APP.get() {
        let _ = app.emit(
            "audio-convert-status",
            Event {
                path,
                status,
                error,
            },
        );
    }
}
fn shared() -> &'static Shared {
    SHARED.get_or_init(|| {
        let state = Arc::new(State::default());
        let worker = Arc::clone(&state);
        let (sender, receiver) = channel::<(String, PathBuf)>();
        std::thread::spawn(move || {
            for (path, target) in receiver {
                let result = if probe::plays_natively(&path) {
                    Ok(path.clone())
                } else if target.is_file()
                    && crate::engine::audio::decode::can_decode_native(&target.to_string_lossy())
                {
                    Ok(target.to_string_lossy().into_owned())
                } else {
                    let _ = std::fs::remove_file(&target);
                    emit(&path, "preparing", None);
                    ffmpeg::convert(&path, &target).and_then(|()| {
                        if cache::target_for(&path).as_ref() != Some(&target) {
                            let _ = std::fs::remove_file(&target);
                            return Err("audio_convert_changed".to_string());
                        }
                        cache::cleanup(&target);
                        Ok(target.to_string_lossy().into_owned())
                    })
                };
                let status = match &result {
                    Ok(ready) => Status::Ready(ready.clone()),
                    Err(error) => Status::Failed(error.clone()),
                };
                worker.finish(target, status);
                if let Err(error) = &result {
                    emit(&path, "failed", Some(error));
                } else if result.as_ref().is_ok_and(|ready| ready != &path) {
                    emit(&path, "ready", None);
                }
            }
        });
        Shared { state, sender }
    })
}
fn request(path: &str) -> Result<(&'static Shared, PathBuf), String> {
    let target = cache::target_for(path).ok_or("audio_file_not_found")?;
    let shared = shared();
    if shared.state.insert_pending(&target)
        && shared
            .sender
            .send((path.to_string(), target.clone()))
            .is_err()
    {
        shared.state.finish(
            target.clone(),
            Status::Failed("audio_convert_failed".into()),
        );
    }
    Ok((shared, target))
}
pub fn prepare(path: &str) -> Result<(), String> {
    let (shared, target) = request(path)?;
    let entries = shared.state.entries.lock().unwrap();
    match entries.get(&target) {
        Some(Status::Failed(error)) => Err(error.clone()),
        _ => Ok(()),
    }
}
pub fn resolve(path: &str, wait: bool) -> Result<String, String> {
    let (shared, target) = request(path)?;
    let entries = shared.state.entries.lock().unwrap();
    let entries = if wait {
        shared
            .state
            .changed
            .wait_timeout_while(entries, Duration::from_secs(600), |map| {
                matches!(map.get(&target), Some(Status::Pending))
            })
            .unwrap()
            .0
    } else {
        entries
    };
    match entries.get(&target) {
        Some(Status::Ready(path)) => Ok(path.clone()),
        Some(Status::Failed(error)) => Err(error.clone()),
        _ => Err("audio_converting".into()),
    }
}
#[cfg(test)]
#[path = "worker_tests.rs"]
mod tests;
