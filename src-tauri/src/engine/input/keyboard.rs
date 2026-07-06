use crate::engine::persist::config_io as config;
use crate::{cmd_button_playback, AppState};
/// Modulo: global_shortcuts.rs
/// Proposito: registrar y ejecutar atajos globales desde Rust/Tauri.
use std::collections::HashSet;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Instala el plugin y conecta un manejador unico para todos los atajos.
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                if let Err(e) = handle_pressed(app, shortcut) {
                    eprintln!("Error en atajo global: {e}");
                }
            }
        })
        .build()
}

/// Sincroniza el registro nativo con la configuracion actual del perfil activo.
pub fn sync(app: &AppHandle) -> Result<(), String> {
    let keys = {
        let state = app.state::<AppState>();
        let cfg = state.config.lock().unwrap();
        collect_keys(&cfg)
    };
    let manager = app.global_shortcut();
    manager.unregister_all().map_err(|e| e.to_string())?;
    for key in keys {
        let shortcut = parse_shortcut(&key)?;
        manager.register(shortcut).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn handle_pressed(app: &AppHandle, shortcut: &Shortcut) -> Result<(), String> {
    let state = app.state::<AppState>();
    let action = find_action(&state, shortcut)?;
    match action {
        ShortcutAction::StopAll => state.audio.lock().unwrap().stop_all(),
        ShortcutAction::Cycle(offset) => {
            cycle_paleta(&state, offset)?;
            app.emit("global-shortcut-refresh", ())
                .map_err(|e| e.to_string())?;
        }
        ShortcutAction::SetPaleta(id) => {
            set_paleta(&state, id)?;
            app.emit("global-shortcut-refresh", ())
                .map_err(|e| e.to_string())?;
        }
        ShortcutAction::PlayButton(id) => cmd_button_playback::play_button_id(&state, &id)?,
    }
    Ok(())
}

enum ShortcutAction {
    StopAll,
    Cycle(i32),
    SetPaleta(String),
    PlayButton(String),
}

fn find_action(state: &AppState, shortcut: &Shortcut) -> Result<ShortcutAction, String> {
    let cfg = state.config.lock().unwrap();
    let profile = cfg
        .profiles
        .iter()
        .find(|p| p.id == cfg.active_profile_id)
        .ok_or("Perfil activo no encontrado")?;
    if !profile.audio.global_keys {
        return Err("Atajos globales desactivados".to_string());
    }

    if matches_key(&profile.audio.key_stop, shortcut) {
        return Ok(ShortcutAction::StopAll);
    }
    if matches_key(&profile.audio.key_next, shortcut) {
        return Ok(ShortcutAction::Cycle(1));
    }
    if matches_key(&profile.audio.key_prev, shortcut) {
        return Ok(ShortcutAction::Cycle(-1));
    }
    for paleta in &profile.paletas {
        if matches_key(&paleta.shortcut, shortcut) {
            return Ok(ShortcutAction::SetPaleta(paleta.id.clone()));
        }
    }
    if let Some(active) = profile
        .paletas
        .iter()
        .find(|p| p.id == profile.active_paleta_id)
    {
        for btn in &active.botones {
            if matches_key(&btn.shortcut, shortcut) {
                return Ok(ShortcutAction::PlayButton(btn.id.clone()));
            }
        }
    }
    Err("Atajo no encontrado".to_string())
}

fn collect_keys(cfg: &crate::model::AppConfig) -> Vec<String> {
    let Some(profile) = cfg.profiles.iter().find(|p| p.id == cfg.active_profile_id) else {
        return Vec::new();
    };
    if !profile.audio.global_keys {
        return Vec::new();
    }

    let mut keys = HashSet::new();
    add_key(&mut keys, &profile.audio.key_stop);
    add_key(&mut keys, &profile.audio.key_next);
    add_key(&mut keys, &profile.audio.key_prev);
    for paleta in &profile.paletas {
        add_key(&mut keys, &paleta.shortcut);
        for btn in &paleta.botones {
            add_key(&mut keys, &btn.shortcut);
        }
    }
    keys.into_iter().collect()
}

fn add_key(keys: &mut HashSet<String>, key: &str) {
    if !key.trim().is_empty() {
        keys.insert(key.trim().to_string());
    }
}

fn matches_key(saved: &str, shortcut: &Shortcut) -> bool {
    if saved.trim().is_empty() {
        return false;
    }
    parse_shortcut(saved)
        .map(|s| &s == shortcut)
        .unwrap_or(false)
}

fn parse_shortcut(key: &str) -> Result<Shortcut, String> {
    normalize_shortcut(key)
        .parse::<Shortcut>()
        .map_err(|e| format!("Atajo invalido {key}: {e}"))
}

fn normalize_shortcut(key: &str) -> String {
    key.trim()
        .replace("Ctrl+", "Control+")
        .replace("Espacio", "Space")
}

fn cycle_paleta(state: &AppState, offset: i32) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
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
    profile.active_paleta_id = profile.paletas[(current + offset).rem_euclid(len) as usize]
        .id
        .clone();
    config::save_config(&cfg)?;
    drop(cfg);
    crate::engine::cache::warm::warm_visible_tab(state);
    Ok(())
}

fn set_paleta(state: &AppState, paleta_id: String) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    if profile.paletas.iter().any(|p| p.id == paleta_id) {
        profile.active_paleta_id = paleta_id;
        config::save_config(&cfg)?;
        drop(cfg);
        crate::engine::cache::warm::warm_visible_tab(state);
    }
    Ok(())
}
