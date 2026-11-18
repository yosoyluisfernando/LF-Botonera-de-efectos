//! Estado de salidas independientes; decisiones y reconexión en Rust.
use super::AppState;
use crate::engine::console::tab_outputs::{self, OutputStatus};

#[tauri::command]
pub async fn get_tab_output_status(
    retry: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<OutputStatus, String> {
    tab_outputs::apply(&state.config, &state.console, retry.unwrap_or(false))
}

pub fn apply(state: &AppState) -> Result<(), String> {
    tab_outputs::apply(&state.config, &state.console, false).map(|_| ())
}

pub fn add_strips(state: &AppState, view: &mut super::cmd_console::ConsoleView) {
    if cfg!(target_os = "android") {
        return;
    }
    let mut lanes: Vec<_> = state.console.tab_lanes().into_iter().collect();
    lanes.sort_by_key(|(_, lane)| *lane);
    for (device, lane) in lanes {
        let bus = crate::engine::console::BusId::Salida(lane);
        view.strips.push(super::cmd_console::StripView {
            bus: format!("salida{lane}"),
            device_name: Some(crate::engine::console::device::display_name(&device)),
            device: Some(device),
            available: state.console.bus(bus).is_some(),
            follows_master: true,
            in_program: false,
            fader: state.console.fader(bus),
            max: 1.0,
        });
    }
}

pub fn finish_config(cfg: std::sync::MutexGuard<'_, crate::model::AppConfig>, state: &AppState) -> Result<crate::model::AppConfig, String> {
    let next = cfg.clone(); drop(cfg);
    apply(state)?;
    Ok(next)
}

#[tauri::command]
pub async fn get_tab_output_devices() -> Vec<crate::engine::console::device::OutputDevice> {
    if cfg!(target_os = "android") { return vec![]; }
    crate::engine::console::device::refresh_outputs()
}
