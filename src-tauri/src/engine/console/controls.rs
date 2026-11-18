//! Estado compartido de los controles, independiente de reproducción y medidores.
use super::{BusId, ConsoleEngine};
use crate::model::AppConfig;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager};
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MasterVolumeState {
    pub volume: f32,
    pub remember: bool,
    pub boost: bool,
    pub max: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FaderState {
    pub value: f32,
    pub max: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VolumeControls {
    pub master: MasterVolumeState,
    pub player_volume: f32,
    pub faders: BTreeMap<String, FaderState>,
}
pub struct ControlTaps {
    values: Vec<(BusId, Arc<AtomicU32>)>,
}
impl ControlTaps {
    pub fn new(console: &ConsoleEngine) -> Self {
        Self {
            values: BusId::every()
                .map(|id| (id, console.volume_handle(id)))
                .collect(),
        }
    }
    pub fn snapshot(&self, cfg: &AppConfig) -> Result<VolumeControls, String> {
        let audio = cfg.active_audio().ok_or("active_profile_not_found")?;
        let maximum = if audio.master_volume_boost { 1.5 } else { 1.0 };
        let faders: BTreeMap<_, _> = self
            .values
            .iter()
            .map(|(bus, handle)| {
                let max = if *bus == BusId::Programa {
                    maximum
                } else {
                    1.0
                };
                let value = f32::from_bits(handle.load(Ordering::Relaxed)).clamp(0.0, max);
                (name(*bus), FaderState { value, max })
            })
            .collect();
        Ok(VolumeControls {
            master: MasterVolumeState {
                volume: faders["programa"].value,
                remember: audio.master_volume_remember,
                boost: audio.master_volume_boost,
                max: maximum,
            },
            player_volume: faders["reproductor"].value,
            faders,
        })
    }
}
fn name(bus: BusId) -> String {
    match bus {
        BusId::Programa => "programa".into(),
        BusId::Efectos => "efectos".into(),
        BusId::Panel => "panel".into(),
        BusId::Cue => "cue".into(),
        BusId::Reproductor => "reproductor".into(),
        BusId::Salida(i) => format!("salida{i}"),
    }
}
/// Solo emite cambios; sigue observando en pausa/reposo y no modifica el audio.
pub fn start(app: tauri::AppHandle, config: Arc<Mutex<AppConfig>>, console: Arc<ConsoleEngine>) {
    std::thread::spawn(move || {
        let taps = ControlTaps::new(&console);
        let mut previous = None;
        loop {
            if let Ok(next) = taps.snapshot(&config.lock().unwrap()) {
                if previous.as_ref() != Some(&next) {
                    let _ = app.emit("volume-controls-changed", &next);
                    previous = Some(next);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            if app.get_webview_window("main").is_none() {
                break;
            }
        }
    });
}
