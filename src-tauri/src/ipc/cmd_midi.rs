use super::AppState;
use crate::engine::input::midi_ports::selected_refs;
use crate::engine::input::midi_rules;
use crate::engine::persist::config_io;
use crate::model::{AppConfig, MidiBinding};
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
pub struct MidiCapture {
    pub binding: MidiBinding,
    pub display: String,
}

#[tauri::command]
pub fn midi_devices(
    state: tauri::State<AppState>,
) -> Vec<crate::engine::input::midi_ports::MidiInputDevice> {
    let cfg = state.config.lock().unwrap();
    state.midi.devices(&cfg.midi)
}

#[tauri::command]
pub fn midi_set_config(
    enabled: bool,
    input_ids: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let previous = cfg.midi.inputs.clone();
    cfg.midi.enabled = enabled;
    cfg.midi.inputs = selected_refs(&input_ids)
        .into_iter()
        .map(|mut item| {
            if item.name == item.id {
                if let Some(old) = previous.iter().find(|old| old.id == item.id) {
                    item.name = old.name.clone();
                }
            }
            item
        })
        .collect();
    config_io::save_config(&cfg)?;
    let next = cfg.clone();
    drop(cfg);
    state.midi.sync();
    Ok(next)
}

#[tauri::command]
pub async fn midi_capture_next(state: tauri::State<'_, AppState>) -> Result<MidiCapture, String> {
    let wait = state.midi.begin_capture()?;
    let event = tauri::async_runtime::spawn_blocking(move || wait.wait(Duration::from_secs(20)))
        .await
        .map_err(|error| format!("midi_capture_task_failed:{error}"))??;
    let binding = MidiBinding {
        device_id: event.device_id.clone(),
        device_name: event.device_name.clone(),
        message: event.message.clone(),
        channel: event.channel,
        number: event.number,
    };
    Ok(MidiCapture {
        binding,
        display: event.display(),
    })
}

#[tauri::command]
pub fn midi_capture_cancel(state: tauri::State<AppState>) {
    state.midi.cancel_capture();
}

pub fn validate_global(
    cfg: &AppConfig,
    key: &str,
    binding: Option<&MidiBinding>,
) -> Result<(), String> {
    if let Some(value) = binding {
        midi_rules::validate_global(cfg, key, value)?;
    }
    Ok(())
}
