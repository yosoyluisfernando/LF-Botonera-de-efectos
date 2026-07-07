/// Modulo: cmd_button_flags.rs
/// Proposito: cambios atomicos de banderas de reproduccion por boton.
use super::AppState;
use crate::ipc::cmd_grid::{active_paleta, save_grid};
use crate::model::grid::GridState;

#[tauri::command]
pub fn toggle_button_flag(
    index: u32,
    flag: String,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    let btn = paleta
        .botones
        .iter_mut()
        .find(|b| b.index == index)
        .ok_or("button_not_found")?;

    match flag.as_str() {
        "loop_mode" => btn.loop_mode = !btn.loop_mode,
        "overlap" => btn.overlap = !btn.overlap,
        "stop_other" => btn.stop_other = !btn.stop_other,
        "restart" => btn.restart = !btn.restart,
        _ => return Err("invalid_button_flag".to_string()),
    }

    save_grid(&mut cfg)
}
