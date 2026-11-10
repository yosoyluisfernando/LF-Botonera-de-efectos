/// Modulo: cmd_history.rs
/// Proposito: comandos IPC para deshacer y rehacer cambios de configuracion.
use super::AppState;
use crate::engine::input::keyboard as global_shortcuts;
use crate::engine::persist::config_io as config;
use crate::model::AppConfig;

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
    let obs_changed = crate::domain::obs::connection_changed(&cfg.obs, &next.obs);
    *cfg = next;
    config::save_config(&cfg)?;
    let out = cfg.clone();
    drop(cfg);
    if !cfg!(target_os = "android") && obs_changed {
        state.obs.configure(out.obs.clone(), None)?;
    }
    global_shortcuts::sync(&app)?;
    Ok(out)
}
