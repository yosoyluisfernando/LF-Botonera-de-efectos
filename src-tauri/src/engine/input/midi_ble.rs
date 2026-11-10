use super::midi::MidiRuntimeMsg;
use super::midi_ble_connection::{self, SERVICE};
use super::midi_ports::MidiPortInfo;
use crate::core::AppState;
use crate::model::MidiConfig;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager as BluetoothManager, Peripheral};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;
use tokio::time::{sleep, timeout};

const PREFIX: &str = "ble-desktop:";

#[derive(Clone, Default)]
pub struct BleMidi {
    state: Arc<Mutex<Devices>>,
    scan: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Default)]
struct Devices {
    found: BTreeMap<String, (String, Peripheral)>,
    workers: HashMap<String, watch::Sender<bool>>,
    connected: HashSet<String>,
}

impl BleMidi {
    pub fn ports(&self) -> Vec<MidiPortInfo> {
        self.state
            .lock()
            .unwrap()
            .found
            .iter()
            .map(|(id, (name, _))| MidiPortInfo {
                id: id.clone(),
                name: name.clone(),
                index: 0,
                #[cfg(target_os = "linux")]
                native_id: String::new(),
            })
            .collect()
    }

    pub fn connected(&self) -> Vec<String> {
        self.state
            .lock()
            .unwrap()
            .connected
            .iter()
            .cloned()
            .collect()
    }

    pub fn restore(&self, app: &AppHandle, config: &MidiConfig) {
        if config.enabled
            && config
                .inputs
                .iter()
                .any(|input| input.id.starts_with(PREFIX))
        {
            let app = app.clone();
            let ble = self.clone();
            tauri::async_runtime::spawn(async move {
                let _ = ble.discover(&app).await;
            });
        }
    }

    pub(super) fn select(&self, app: &AppHandle, config: &MidiConfig, tx: &Sender<MidiRuntimeMsg>) {
        let mut devices = self.state.lock().unwrap();
        let selected = config
            .inputs
            .iter()
            .map(|item| item.id.as_str())
            .collect::<HashSet<_>>();
        for (id, control) in &devices.workers {
            let desired = config.enabled && selected.contains(id.as_str());
            control.send_if_modified(|current| {
                if *current == desired {
                    return false;
                }
                *current = desired;
                true
            });
        }
        if !config.enabled {
            return;
        }
        for id in selected {
            if devices.workers.contains_key(id) {
                continue;
            }
            let Some((name, peripheral)) = devices.found.get(id).cloned() else {
                continue;
            };
            let (control, enabled) = watch::channel(true);
            devices.workers.insert(id.into(), control);
            let app = app.clone();
            let ble = self.clone();
            let id = id.to_string();
            let status_id = id.clone();
            let tx = tx.clone();
            tauri::async_runtime::spawn(midi_ble_connection::run(
                peripheral,
                id,
                name,
                enabled,
                tx,
                move |online| {
                    let changed = {
                        let mut state = ble.state.lock().unwrap();
                        if online {
                            state.connected.insert(status_id.clone())
                        } else {
                            state.connected.remove(&status_id)
                        }
                    };
                    if changed {
                        let _ = app.emit("midi-devices-changed", ());
                    }
                },
            ));
        }
    }

    pub async fn discover(&self, app: &AppHandle) -> Result<usize, String> {
        let _guard = self.scan.try_lock().map_err(|_| "bluetooth_busy")?;
        let mut adapters = start_scans().await?;
        sleep(Duration::from_secs(6)).await;
        let mut found = BTreeMap::new();
        while let Some(adapter) = adapters.pop() {
            let _ = timeout(Duration::from_secs(3), adapter.stop_scan()).await;
            let Ok(Ok(peripherals)) = timeout(Duration::from_secs(3), adapter.peripherals()).await
            else {
                continue;
            };
            for peripheral in peripherals {
                let Ok(Ok(Some(properties))) =
                    timeout(Duration::from_millis(500), peripheral.properties()).await
                else {
                    continue;
                };
                if properties.services.contains(&SERVICE) {
                    let address = peripheral.address().to_string();
                    let name = properties
                        .local_name
                        .filter(|name| !name.is_empty())
                        .unwrap_or_else(|| address.clone());
                    found.insert(format!("{PREFIX}{address}"), (name, peripheral));
                }
            }
        }
        let count = found.len();
        {
            let mut state = self.state.lock().unwrap();
            let active = state.connected.clone();
            state
                .found
                .retain(|id, _| active.contains(id) || found.contains_key(id));
            state.found.extend(found);
        }
        app.state::<AppState>().midi.sync();
        let _ = app.emit("midi-devices-changed", ());
        Ok(count)
    }
}

async fn start_scans() -> Result<Vec<Adapter>, String> {
    let adapters = timeout(Duration::from_secs(5), async {
        BluetoothManager::new().await?.adapters().await
    })
    .await
    .map_err(|_| "bluetooth_scan_failed")?
    .map_err(|_| "bluetooth_unavailable")?;
    if adapters.is_empty() {
        return Err("bluetooth_unavailable".into());
    }
    let mut running = Vec::new();
    for adapter in adapters {
        let filter = ScanFilter {
            services: vec![SERVICE],
        };
        if matches!(
            timeout(Duration::from_secs(4), adapter.start_scan(filter)).await,
            Ok(Ok(()))
        ) {
            running.push(adapter);
        }
    }
    if running.is_empty() {
        return Err("bluetooth_scan_failed".into());
    }
    Ok(running)
}
