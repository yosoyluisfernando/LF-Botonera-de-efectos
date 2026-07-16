//! Modulo: cmd_player_config.rs
//! Proposito: los ajustes del reproductor auxiliar que SE PERSISTEN (modo de
//! avance, volumen, dispositivo y formato del contador). El transporte, que es
//! estado de ejecucion y no se guarda, vive en `cmd_player.rs`.
use super::AppState;
use crate::domain::player::PlayerMode;
use crate::engine::persist::config_io;

/// Alterna el contador entre tiempo transcurrido y restante. Lo decide Rust y se
/// recuerda entre sesiones; la UI solo pide el cambio y pinta lo que le llega.
#[tauri::command]
pub fn player_toggle_time_display(state: tauri::State<AppState>) -> Result<String, String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.player.time_display = if cfg.player.time_display == "remaining" {
        "elapsed".into()
    } else {
        "remaining".into()
    };
    let value = cfg.player.time_display.clone();
    config_io::save_config(&cfg)?;
    Ok(value)
}

#[tauri::command]
pub fn player_set_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    let parsed = PlayerMode::parse(&mode)?;
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.player.playback_mode = mode;
        config_io::save_config(&cfg)?;
    }
    state.player.lock().unwrap().set_mode(parsed);
    Ok(())
}

/// Volumen propio del reproductor. `persist` = false mientras se arrastra el
/// deslizador: aplicarlo es un atomico (instantaneo), pero guardar en disco en
/// cada pixel seria una tormenta de escrituras. Al soltar se llama con `true`.
/// Sin `persist` (None) guarda: asi lo usan los ajustes, que aplican al Guardar.
#[tauri::command]
pub fn player_set_volume(
    volume: f32,
    persist: Option<bool>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if !volume.is_finite() || !(0.0..=1.5).contains(&volume) {
        return Err("invalid_volume".into());
    }
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.player.volume = volume;
        if persist.unwrap_or(true) {
            config_io::save_config(&cfg)?;
        }
    }
    state.player.lock().unwrap().set_volume(volume);
    Ok(())
}

/// Fija el dispositivo de salida propio. "" = el mismo de los efectos.
#[tauri::command]
pub fn player_set_device(device: String, state: tauri::State<AppState>) -> Result<(), String> {
    let resolved = {
        let mut cfg = state.config.lock().unwrap();
        cfg.player.output_device = device.clone();
        config_io::save_config(&cfg)?;
        if device.is_empty() {
            cfg.active_audio()
                .map(|a| a.out_main.clone())
                .unwrap_or_else(|| "default".into())
        } else {
            device
        }
    };
    state.player.lock().unwrap().set_device(&resolved);
    Ok(())
}

/// Que hacer al soltar una carpeta con muchas canciones: "ask" | "always" |
/// "never". El aviso lo guarda con su check "recordar siempre"; este comando
/// existe para poder cambiarlo despues desde Ajustes, por si se respondio mal.
#[tauri::command]
pub fn player_set_large_folder_action(
    action: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if !crate::model::player::LARGE_FOLDER_ACTIONS.contains(&action.as_str()) {
        return Err("invalid_large_folder_action".into());
    }
    let mut cfg = state.config.lock().unwrap();
    cfg.player.large_folder_action = action;
    config_io::save_config(&cfg)
}
