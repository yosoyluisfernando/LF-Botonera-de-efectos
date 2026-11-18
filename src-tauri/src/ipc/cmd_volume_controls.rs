//! Una única lectura autoritativa para todas las representaciones del volumen.
use super::AppState;
use crate::engine::console::controls::{ControlTaps, VolumeControls};
#[tauri::command]
pub fn get_volume_controls(state: tauri::State<AppState>) -> Result<VolumeControls, String> {
    ControlTaps::new(&state.console).snapshot(&state.config.lock().unwrap())
}
