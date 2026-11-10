use super::{events::publish, sync, transport::Session, Command};
use crate::model::obs::{ObsConfig, ObsSnapshot};
use crate::model::AppConfig;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::AppHandle;
pub fn run(
    app: AppHandle,
    rx: Receiver<Command>,
    shared: Arc<Mutex<ObsSnapshot>>,
    generation: Arc<AtomicU64>,
    busy: Arc<AtomicBool>,
    config: Arc<Mutex<AppConfig>>,
) {
    let mut cfg = ObsConfig::default();
    let mut password = String::new();
    let mut session = None;
    let mut current_ticket = 0;
    let mut retry = Some(Instant::now());
    let mut last_event = String::new();
    let mut refresh = Instant::now();
    loop {
        match rx.recv_timeout(if cfg.enabled {
            Duration::from_millis(40)
        } else {
            Duration::from_secs(3600)
        }) {
            Ok(Command::Configure(ticket, next, secret)) => {
                current_ticket = ticket;
                cfg = next;
                if let Some(secret) = secret {
                    password = secret;
                }
                session = None;
                retry = Some(Instant::now());
                let state = shared.lock().unwrap().clone();
                publish(
                    &app,
                    &shared,
                    state,
                    current_ticket,
                    &generation,
                    &busy,
                    &config,
                    &mut last_event,
                );
            }
            Ok(Command::Execute(ticket, id, action)) => {
                if ticket != generation.load(Ordering::SeqCst) {
                    continue;
                }
                let mut state = shared.lock().unwrap().clone();
                publish(
                    &app,
                    &shared,
                    state.clone(),
                    ticket,
                    &generation,
                    &busy,
                    &config,
                    &mut last_event,
                );
                let result = if let Some(s) = session.as_mut() {
                    // Actualizar antes de alternar: OBS es la fuente de verdad.
                    sync::refresh(s, &mut state)
                        .and_then(|_| {
                            if ticket != generation.load(Ordering::SeqCst) {
                                return Err("obs_disabled".into());
                            }
                            sync::execute(s, &action, &state)
                        })
                        .and_then(|_| sync::refresh(s, &mut state))
                } else {
                    Err("obs_disconnected".into())
                };
                if ticket == generation.load(Ordering::SeqCst) {
                    busy.store(false, Ordering::SeqCst);
                }
                state.busy = false;
                state.pending_id.clear();
                if let Err(error) = result {
                    state.failed_id = id;
                    state.error = error.clone();
                    if matches!(
                        error.as_str(),
                        "obs_disconnected" | "obs_timeout" | "obs_protocol_error"
                    ) {
                        session = None;
                        state.connected = false;
                        state.status = "disconnected".into();
                        retry = cfg
                            .reconnect
                            .then(|| Instant::now() + Duration::from_secs(3));
                    }
                }
                publish(
                    &app,
                    &shared,
                    state,
                    ticket,
                    &generation,
                    &busy,
                    &config,
                    &mut last_event,
                );
            }
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
        if !cfg.enabled || current_ticket != generation.load(Ordering::SeqCst) {
            continue;
        }
        let ticket = current_ticket;
        if session.is_none() && retry.is_some_and(|at| Instant::now() >= at) {
            let mut state = ObsSnapshot {
                status: "connecting".into(),
                ..Default::default()
            };
            publish(
                &app,
                &shared,
                state.clone(),
                ticket,
                &generation,
                &busy,
                &config,
                &mut last_event,
            );
            match Session::connect(&cfg, &password).and_then(|mut s| {
                sync::refresh(&mut s, &mut state)?;
                Ok(s)
            }) {
                Ok(s) => {
                    session = Some(s);
                    state.connected = true;
                    state.status = "connected".into();
                }
                Err(error) => {
                    state.status = "disconnected".into();
                    state.error = error;
                    retry = cfg
                        .reconnect
                        .then(|| Instant::now() + Duration::from_secs(3));
                }
            }
            publish(
                &app,
                &shared,
                state,
                ticket,
                &generation,
                &busy,
                &config,
                &mut last_event,
            );
            refresh = Instant::now() + Duration::from_secs(2);
        }
        if let Some(s) = session.as_mut() {
            let mut state = shared.lock().unwrap().clone();
            let result = s.read().and_then(|event| {
                if Instant::now() >= refresh || event.is_some_and(|e| e["op"] == 5) {
                    sync::refresh(s, &mut state)?;
                    refresh = Instant::now() + Duration::from_secs(2);
                }
                Ok(())
            });
            if let Err(error) = result {
                session = None;
                state.connected = false;
                if ticket == generation.load(Ordering::SeqCst) {
                    busy.store(false, Ordering::SeqCst);
                }
                state.busy = false;
                state.pending_id.clear();
                state.status = "disconnected".into();
                state.error = error;
                retry = cfg
                    .reconnect
                    .then(|| Instant::now() + Duration::from_secs(3));
            }
            publish(
                &app,
                &shared,
                state,
                ticket,
                &generation,
                &busy,
                &config,
                &mut last_event,
            );
        }
    }
}
