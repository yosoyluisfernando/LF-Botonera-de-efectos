use super::{incremental, indexer, root_store};
use crate::engine::persist::db;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

enum Message {
    Event(notify::Result<Event>),
    ReconcileAll,
    Stop,
}

pub struct LibraryMonitor {
    watcher: Option<RecommendedWatcher>,
    watched: Vec<PathBuf>,
    sender: mpsc::Sender<Message>,
    worker: Option<JoinHandle<()>>,
    worker_done: mpsc::Receiver<()>,
}

pub fn start(
    database_path: PathBuf,
    operation: Arc<Mutex<()>>,
    roots: &[root_store::LibraryRoot],
) -> Result<(LibraryMonitor, Option<String>), String> {
    let (sender, receiver) = mpsc::channel();
    let (done_sender, worker_done) = mpsc::channel();
    let callback_sender = sender.clone();
    let watcher = notify::recommended_watcher(move |event| {
        let _ = callback_sender.send(Message::Event(event));
    })
    .map_err(|error| error.to_string())?;
    let worker = std::thread::spawn(move || {
        worker_loop(database_path, operation, receiver);
        let _ = done_sender.send(());
    });
    let mut monitor = LibraryMonitor {
        watcher: Some(watcher),
        watched: Vec::new(),
        sender,
        worker: Some(worker),
        worker_done,
    };
    let refresh_error = monitor.refresh(roots).err();
    let _ = monitor.sender.send(Message::ReconcileAll);
    Ok((monitor, refresh_error))
}

impl LibraryMonitor {
    pub fn refresh(&mut self, roots: &[root_store::LibraryRoot]) -> Result<(), String> {
        let watcher = self.watcher.as_mut().ok_or("library_monitor_stopped")?;
        let mut first_error = None;
        for path in self.watched.drain(..) {
            let _ = watcher.unwatch(&path);
        }
        for root in roots.iter().filter(|root| root.enabled) {
            let path = PathBuf::from(&root.path);
            if !path.is_dir() {
                continue;
            }
            match watcher.watch(&path, RecursiveMode::Recursive) {
                Ok(()) => self.watched.push(path),
                Err(error) if first_error.is_none() => first_error = Some(error.to_string()),
                Err(_) => {}
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for LibraryMonitor {
    fn drop(&mut self) {
        self.watcher.take();
        let _ = self.sender.send(Message::Stop);
        if self
            .worker_done
            .recv_timeout(Duration::from_millis(500))
            .is_ok()
        {
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }
}

fn worker_loop(
    database_path: PathBuf,
    operation: Arc<Mutex<()>>,
    receiver: mpsc::Receiver<Message>,
) {
    while let Ok(message) = receiver.recv() {
        match message {
            Message::Stop => break,
            Message::ReconcileAll => reconcile_all(&database_path, &operation),
            Message::Event(Ok(event)) if !matches!(event.kind, EventKind::Access(_)) => {
                let (paths, reconcile_all, stop) = collect_burst(event.paths, &receiver);
                if stop {
                    break;
                }
                apply_burst(&database_path, &operation, &paths, reconcile_all);
            }
            Message::Event(Err(_)) => reconcile_all(&database_path, &operation),
            Message::Event(Ok(_)) => {}
        }
    }
}

fn collect_burst(
    mut paths: Vec<PathBuf>,
    receiver: &mpsc::Receiver<Message>,
) -> (Vec<PathBuf>, bool, bool) {
    let mut reconcile = false;
    let mut stop = false;
    loop {
        match receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(Message::Event(Ok(event))) if !matches!(event.kind, EventKind::Access(_)) => {
                paths.extend(event.paths);
            }
            Ok(Message::Event(Err(_))) | Ok(Message::ReconcileAll) => reconcile = true,
            Ok(Message::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                stop = true;
                break;
            }
            Ok(Message::Event(Ok(_))) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => break,
        }
    }
    (paths, reconcile, stop)
}

fn apply_burst(
    database_path: &std::path::Path,
    operation: &Mutex<()>,
    paths: &[PathBuf],
    force_reconcile: bool,
) {
    let Ok(_guard) = operation.lock() else {
        return;
    };
    let Ok(mut connection) = db::open(Some(database_path)) else {
        return;
    };
    let Ok(report) = incremental::apply_paths(&mut connection, paths) else {
        reconcile_connection(&mut connection);
        return;
    };
    for root_id in report.reconcile_roots {
        let _ = indexer::sync_root(&mut connection, root_id, |_| {});
    }
    if force_reconcile {
        reconcile_connection(&mut connection);
    }
}

fn reconcile_all(database_path: &std::path::Path, operation: &Mutex<()>) {
    let Ok(_guard) = operation.lock() else {
        return;
    };
    let Ok(mut connection) = db::open(Some(database_path)) else {
        return;
    };
    reconcile_connection(&mut connection);
}

fn reconcile_connection(connection: &mut rusqlite::Connection) {
    let roots = root_store::list(connection).unwrap_or_default();
    for root in roots.into_iter().filter(|root| root.enabled) {
        let _ = indexer::sync_root(connection, root.id, |_| {});
    }
}

#[cfg(test)]
#[path = "monitor_tests.rs"]
mod tests;
