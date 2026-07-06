/// Modulo: cmd_history.rs
/// Proposito: comandos IPC para deshacer y rehacer cambios de configuracion.
use super::AppState;
use crate::{config, global_shortcuts, model::AppConfig};

#[tauri::command]
pub fn undo_config(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    restore(app, state, true)
}

#[tauri::command]
pub fn redo_config(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    restore(app, state, false)
}

fn restore(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    undo: bool,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let next = {
        let mut history = state.history.lock().unwrap();
        if undo {
            history.undo(&cfg)
        } else {
            history.redo(&cfg)
        }
    }
    .ok_or(if undo {
        "nothing_to_undo"
    } else {
        "nothing_to_redo"
    })?;
    *cfg = next;
    config::save_config(&cfg)?;
    let out = cfg.clone();
    drop(cfg);
    global_shortcuts::sync(&app)?;
    Ok(out)
}
