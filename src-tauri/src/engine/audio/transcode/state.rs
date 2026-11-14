//! Estados compartidos; una petición se publica antes de ponerla en cola.
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Status {
    Pending,
    Ready(String),
    Failed(String),
}
#[derive(Default)]
pub(super) struct State {
    pub entries: Mutex<HashMap<PathBuf, Status>>,
    pub changed: Condvar,
}
impl State {
    pub fn insert_pending(&self, target: &PathBuf) -> bool {
        let mut entries = self.entries.lock().unwrap();
        if matches!(entries.get(target), Some(Status::Ready(path)) if !std::path::Path::new(path).is_file())
        {
            entries.remove(target);
        }
        if entries.contains_key(target) {
            return false;
        }
        // No eliminar trabajos en curso; los resultados se pueden reconstruir del disco.
        if entries.len() >= 1024 {
            entries.retain(|_, status| *status == Status::Pending);
        }
        entries.insert(target.clone(), Status::Pending);
        true
    }
    pub fn finish(&self, target: PathBuf, status: Status) {
        self.finish_with(target, status, || {});
    }
    pub fn finish_with(&self, target: PathBuf, status: Status, publish: impl FnOnce()) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(target, status);
        publish();
        drop(entries);
        self.changed.notify_all();
    }
}
