/// Modulo: cmd_button_types.rs
/// Proposito: expone al frontend los tipos de boton permitidos.
use super::AppState;
use crate::button_types::{self, ButtonTypeState};

#[tauri::command]
pub fn get_edit_button_types(
    current_type: Option<String>,
    state: tauri::State<AppState>,
) -> ButtonTypeState {
    let cfg = state.config.lock().unwrap();
    button_types::editor_state(&cfg, current_type.as_deref())
}
