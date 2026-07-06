/// Modulo: cmd_local_shortcuts.rs
/// Proposito: resolver atajos capturados por la ventana con foco.
use super::AppState;
use crate::ipc::cmd_button_playback;
use crate::engine::persist::config_io as config;
use crate::model::AppConfig;
use serde::Serialize;

#[derive(Serialize)]
pub struct LocalShortcutResult {
    pub handled: bool,
    pub refresh: bool,
}

/// JS solo captura la tecla; Rust decide que accion corresponde.
#[tauri::command]
pub fn handle_local_shortcut(
    key: String,
    state: tauri::State<AppState>,
) -> Result<LocalShortcutResult, String> {
    let action = {
        let cfg = state.config.lock().unwrap();
        resolve_local_action(&cfg, &key)?
    };

    match action {
        LocalShortcutAction::None => Ok(result(false, false)),
        LocalShortcutAction::StopAll => {
            state.audio.lock().unwrap().stop_all();
            Ok(result(true, false))
        }
        LocalShortcutAction::Cycle(offset) => {
            {
                let mut cfg = state.config.lock().unwrap();
                cycle_active_paleta(&mut cfg, offset)?;
                config::save_config(&cfg)?;
            }
            crate::engine::cache::warm::warm_visible_tab(&state);
            Ok(result(true, true))
        }
        LocalShortcutAction::SetPaleta(id) => {
            {
                let mut cfg = state.config.lock().unwrap();
                set_active_paleta(&mut cfg, &id)?;
                config::save_config(&cfg)?;
            }
            crate::engine::cache::warm::warm_visible_tab(&state);
            Ok(result(true, true))
        }
        LocalShortcutAction::PlayButton(id) => {
            cmd_button_playback::play_button_id(&state, &id)?;
            Ok(result(true, false))
        }
    }
}

enum LocalShortcutAction {
    None,
    StopAll,
    Cycle(i32),
    SetPaleta(String),
    PlayButton(String),
}

fn resolve_local_action(cfg: &AppConfig, key: &str) -> Result<LocalShortcutAction, String> {
    let profile = cfg
        .profiles
        .iter()
        .find(|p| p.id == cfg.active_profile_id)
        .ok_or("Perfil activo no encontrado")?;
    if profile.audio.global_keys {
        return Ok(LocalShortcutAction::None);
    }

    let audio = &profile.audio;
    if same_key(&audio.key_stop, key) {
        return Ok(LocalShortcutAction::StopAll);
    }
    if same_key(&audio.key_next, key) {
        return Ok(LocalShortcutAction::Cycle(1));
    }
    if same_key(&audio.key_prev, key) {
        return Ok(LocalShortcutAction::Cycle(-1));
    }
    for paleta in &profile.paletas {
        if same_key(&paleta.shortcut, key) {
            return Ok(LocalShortcutAction::SetPaleta(paleta.id.clone()));
        }
    }
    if let Some(active) = profile
        .paletas
        .iter()
        .find(|p| p.id == profile.active_paleta_id)
    {
        for btn in &active.botones {
            if same_key(&btn.shortcut, key) {
                return Ok(LocalShortcutAction::PlayButton(btn.id.clone()));
            }
        }
    }
    Ok(LocalShortcutAction::None)
}

fn cycle_active_paleta(cfg: &mut AppConfig, offset: i32) -> Result<(), String> {
    let pid = cfg.active_profile_id.clone();
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    let len = profile.paletas.len() as i32;
    if len == 0 {
        return Err("El perfil no tiene pestanas".to_string());
    }
    let current = profile
        .paletas
        .iter()
        .position(|p| p.id == profile.active_paleta_id)
        .unwrap_or(0) as i32;
    let next = (current + offset).rem_euclid(len) as usize;
    profile.active_paleta_id = profile.paletas[next].id.clone();
    Ok(())
}

fn set_active_paleta(cfg: &mut AppConfig, paleta_id: &str) -> Result<(), String> {
    let pid = cfg.active_profile_id.clone();
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    if profile.paletas.iter().any(|p| p.id == paleta_id) {
        profile.active_paleta_id = paleta_id.to_string();
    }
    Ok(())
}

fn same_key(saved: &str, key: &str) -> bool {
    !saved.trim().is_empty() && saved.trim().eq_ignore_ascii_case(key.trim())
}

fn result(handled: bool, refresh: bool) -> LocalShortcutResult {
    LocalShortcutResult { handled, refresh }
}
