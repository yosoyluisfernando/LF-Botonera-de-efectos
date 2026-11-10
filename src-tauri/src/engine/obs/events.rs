use crate::{
    domain::obs::view,
    model::{obs::ObsSnapshot, AppConfig},
};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use tauri::{AppHandle, Emitter};
pub(super) fn publish(
    app: &AppHandle,
    shared: &Mutex<ObsSnapshot>,
    mut state: ObsSnapshot,
    ticket: u64,
    generation: &AtomicU64,
    busy: &AtomicBool,
    config: &Mutex<AppConfig>,
    last: &mut String,
) {
    let mut stored = shared.lock().unwrap();
    if generation.load(Ordering::SeqCst) != ticket {
        return;
    }
    if busy.load(Ordering::SeqCst) {
        state.busy = true;
        state.pending_id = stored.pending_id.clone();
    }
    *stored = state.clone();
    drop(stored);
    let cfg = config.lock().unwrap();
    let payload = view::view(&cfg, state);
    if let Ok(encoded) = serde_json::to_string(&payload) {
        if *last != encoded {
            let _ = app.emit("obs-state", payload);
            *last = encoded;
        }
    }
}
