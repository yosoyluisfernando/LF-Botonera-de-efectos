/// Módulo: cmd_keys.rs
/// Propósito: Comandos IPC para los atajos globales de teclado (Fase 5):
/// persistencia de las teclas configuradas y navegación cíclica de pestañas.
use super::AppState;
use crate::engine::input::actions as input_actions;
use crate::engine::input::keyboard as global_shortcuts;
use crate::engine::input::profile_shortcuts;
use crate::engine::persist::config_io as config;
use crate::model::{AppConfig, MidiBinding};

/// Guarda las acciones de teclado del perfil, incluido el transporte auxiliar.
/// Los campos opcionales omitidos conservan su asignacion; vacios la eliminan.
#[tauri::command]
pub fn set_global_keys(
    key_stop: String,
    key_next: String,
    key_prev: String,
    key_player_play: Option<String>,
    key_player_stop: Option<String>,
    key_player_next: Option<String>,
    key_mic: Option<String>,
    midi_stop: Option<MidiBinding>,
    midi_next: Option<MidiBinding>,
    midi_prev: Option<MidiBinding>,
    global_keys: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let mut audio = cfg
        .active_profile()
        .ok_or("Perfil activo no encontrado")?
        .audio
        .clone();
    audio.key_stop = key_stop;
    audio.key_next = key_next;
    audio.key_prev = key_prev;
    if let Some(key) = key_player_play {
        audio.key_player_play = key;
    }
    if let Some(key) = key_player_stop {
        audio.key_player_stop = key;
    }
    if let Some(key) = key_player_next {
        audio.key_player_next = key;
    }
    if let Some(key) = key_mic {
        audio.key_mic = key;
    }
    profile_shortcuts::validate(&cfg, &audio)?;
    crate::ipc::cmd_midi::validate_global(&cfg, "stop", midi_stop.as_ref())?;
    crate::ipc::cmd_midi::validate_global(&cfg, "next", midi_next.as_ref())?;
    crate::ipc::cmd_midi::validate_global(&cfg, "prev", midi_prev.as_ref())?;
    let profile = cfg
        .active_profile_mut()
        .ok_or("Perfil activo no encontrado")?;
    profile.audio = audio;
    if let Some(value) = midi_stop {
        profile.audio.midi_stop = value;
    }
    if let Some(value) = midi_next {
        profile.audio.midi_next = value;
    }
    if let Some(value) = midi_prev {
        profile.audio.midi_prev = value;
    }
    if let Some(v) = global_keys {
        profile.audio.global_keys = v;
    }
    config::save_config(&cfg)?;
    drop(cfg);
    global_shortcuts::sync(&app)?;
    state.midi.sync();
    let cfg = state.config.lock().unwrap();
    Ok(cfg.clone())
}

/// Borra el atajo de un botón específico en cualquier paleta del perfil activo.
#[tauri::command]
pub fn clear_button_shortcut(
    paleta_id: String,
    index: u32,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    let profile = cfg
        .active_profile_mut()
        .ok_or("Perfil activo no encontrado")?;
    if let Some(paleta) = profile.paletas.iter_mut().find(|p| p.id == paleta_id) {
        if let Some(btn) = paleta.botones.iter_mut().find(|b| b.index == index) {
            btn.shortcut = String::new();
            config::save_config(&cfg)?;
        }
    }
    drop(cfg);
    global_shortcuts::sync(&app)?;
    let cfg = state.config.lock().unwrap();
    Ok(cfg.clone())
}

/// Cambia la pestaña activa al vecino indicado: offset +1 (siguiente) o
/// -1 (anterior), con comportamiento circular.
#[tauri::command]
pub fn cycle_paleta(offset: i32, state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    input_actions::cycle_paleta(&mut cfg, offset)?;
    config::save_config(&cfg)?;
    let result = cfg.clone();
    drop(cfg);
    crate::engine::cache::warm::warm_visible_tab(&state);
    Ok(result)
}
