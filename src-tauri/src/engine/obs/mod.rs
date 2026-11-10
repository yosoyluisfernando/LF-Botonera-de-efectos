//! Motor OBS independiente: un worker posee la conexiÃ³n; nunca bloquea audio ni UI.
mod events;
mod filters;
mod record;
mod resources;
mod sync;
#[cfg(test)]
mod test_support;
pub(crate) mod transport;
mod worker;
use crate::model::obs::{ObsAction, ObsConfig, ObsSnapshot};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex,
};
use tauri::AppHandle;

pub(super) enum Command {
    Configure(u64, ObsConfig, Option<String>),
    Execute(u64, String, ObsAction),
}
pub struct ObsEngine {
    tx: Mutex<Option<SyncSender<Command>>>,
    pub(super) snapshot: Arc<Mutex<ObsSnapshot>>,
    pub(super) generation: Arc<AtomicU64>,
    busy: Arc<AtomicBool>,
    password: Mutex<String>,
}
impl ObsEngine {
    pub fn new() -> Self {
        Self {
            password: Mutex::new(String::new()),
            busy: Arc::new(AtomicBool::new(false)),
            tx: Mutex::new(None),
            snapshot: Arc::new(Mutex::new(ObsSnapshot::default())),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
    pub fn start(
        &self,
        app: AppHandle,
        config: Arc<Mutex<crate::model::AppConfig>>,
    ) -> Result<(), String> {
        let cfg = config.lock().unwrap().obs.clone();
        let busy = Arc::clone(&self.busy);
        let (tx, rx) = sync_channel(8);
        let snapshot = Arc::clone(&self.snapshot);
        let generation = Arc::clone(&self.generation);
        std::thread::Builder::new()
            .name("obs-control".into())
            .spawn(move || worker::run(app, rx, snapshot, generation, busy, config))
            .map_err(|_| "obs_worker_failed")?;
        *self.tx.lock().unwrap() = Some(tx);
        self.configure(cfg, None)
    }
    pub fn session_password(&self) -> String {
        self.password.lock().unwrap().clone()
    }
    pub fn snapshot(&self) -> ObsSnapshot {
        self.snapshot.lock().unwrap().clone()
    }
    pub fn configure(&self, cfg: ObsConfig, password: Option<String>) -> Result<(), String> {
        let mut state = self.snapshot.lock().unwrap();
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or("obs_worker_failed")?;
        let secret = password.unwrap_or_else(|| self.password.lock().unwrap().clone());
        let ticket = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        if tx
            .try_send(Command::Configure(
                ticket,
                cfg.clone(),
                Some(secret.clone()),
            ))
            .is_err()
        {
            self.generation.fetch_sub(1, Ordering::SeqCst);
            return Err("obs_busy".into());
        }
        *self.password.lock().unwrap() = secret;
        self.busy.store(false, Ordering::SeqCst);
        *state = ObsSnapshot {
            status: if cfg.enabled {
                "connecting"
            } else {
                "disabled"
            }
            .into(),
            ..Default::default()
        };
        Ok(())
    }
    pub fn execute(&self, id: String, a: ObsAction) -> Result<(), String> {
        let mut state = self.snapshot.lock().unwrap();
        if !state.connected {
            return Err("obs_disconnected".into());
        }
        if self.busy.load(Ordering::SeqCst) {
            return Err("obs_busy".into());
        }
        crate::domain::obs::actions::request(&a, &state)?;
        self.tx
            .lock()
            .unwrap()
            .as_ref()
            .ok_or("obs_worker_failed")?
            .try_send(Command::Execute(
                self.generation.load(Ordering::SeqCst),
                id.clone(),
                a,
            ))
            .map_err(|_| "obs_busy")?;
        self.busy.store(true, Ordering::SeqCst);
        state.busy = true;
        state.pending_id = id;
        state.failed_id.clear();
        state.error.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
