use super::profile_shortcuts::{self, ProfileAction};
use crate::core::AppState;
use crate::engine::input::actions as input_actions;
use crate::engine::persist::config_io as config;
use crate::ipc::cmd_button_playback;
/// Modulo: global_shortcuts.rs
/// Proposito: registrar y ejecutar atajos globales desde Rust/Tauri.
use std::collections::HashSet;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Instala el plugin y conecta un manejador unico para todos los atajos.
pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state() == ShortcutState::Released {
                let _ = app
                    .state::<AppState>()
                    .mic
                    .send(crate::engine::mic::Command::Input(
                        format!("global:{}", shortcut.id()),
                        false,
                    ));
            }
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
        ShortcutAction::Profile(action) => {
            if profile_shortcuts::execute(&state, action, &format!("global:{}", shortcut.id()))? {
                app.emit("global-shortcut-refresh", ())
                    .map_err(|e| e.to_string())?;
            }
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ShortcutAction {
    Profile(ProfileAction),
    SetPaleta(String),
    PlayButton(String),
}

fn find_action(state: &AppState, shortcut: &Shortcut) -> Result<ShortcutAction, String> {
    let cfg = state.config.lock().unwrap();
    resolve_action(&cfg, shortcut)
}

pub(crate) fn resolve_action(
    cfg: &crate::model::AppConfig,
    shortcut: &Shortcut,
) -> Result<ShortcutAction, String> {
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    if !profile.audio.global_keys {
        return Err("Atajos globales desactivados".to_string());
    }

    if let Some(action) =
        profile_shortcuts::resolve(&profile.audio, |key| matches_key(key, shortcut))
    {
        return Ok(ShortcutAction::Profile(action));
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
    let fixed = if cfg.fixed_panel.scope == "profile" {
        &profile.fixed_buttons
    } else {
        &cfg.fixed_panel.global_buttons
    };
    for btn in fixed {
        if matches_key(&btn.shortcut, shortcut) {
            return Ok(ShortcutAction::PlayButton(btn.id.clone()));
        }
    }
    Err("Atajo no encontrado".to_string())
}

pub(crate) fn collect_keys(cfg: &crate::model::AppConfig) -> Vec<String> {
    let Some(profile) = cfg.active_profile() else {
        return Vec::new();
    };
    if !profile.audio.global_keys {
        return Vec::new();
    }

    let mut keys = HashSet::new();
    for (_, key) in profile_shortcuts::bindings(&profile.audio) {
        add_key(&mut keys, key);
    }
    for paleta in &profile.paletas {
        add_key(&mut keys, &paleta.shortcut);
        for btn in &paleta.botones {
            add_key(&mut keys, &btn.shortcut);
        }
    }
    let fixed = if cfg.fixed_panel.scope == "profile" {
        &profile.fixed_buttons
    } else {
        &cfg.fixed_panel.global_buttons
    };
    for btn in fixed {
        add_key(&mut keys, &btn.shortcut);
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
    profile_shortcuts::parse(key)
}

fn set_paleta(state: &AppState, paleta_id: String) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    if input_actions::activate_paleta(&mut cfg, &paleta_id)? {
        config::save_config(&cfg)?;
        drop(cfg);
        crate::engine::cache::warm::warm_visible_tab(state);
    }
    Ok(())
}
