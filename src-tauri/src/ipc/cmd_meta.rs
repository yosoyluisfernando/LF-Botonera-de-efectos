/// Modulo: cmd_meta.rs
/// Proposito: comandos IPC de metadatos de la aplicacion.
use crate::core::AppState;
use crate::domain::clock;
use crate::engine::persist::config_io::save_config;
use tauri::State;

pub use clock::start_clock_thread;

/// Devuelve la version del ejecutable compilada desde Cargo.toml.
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Alterna entre formato 24 h y 12 h. Persiste el cambio en disco.
#[tauri::command]
pub fn toggle_clock_format(state: State<AppState>) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.clock_24h = !cfg.clock_24h;
    let new_val = cfg.clock_24h;
    let _ = save_config(&cfg);
    new_val
}
