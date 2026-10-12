/// Modulo: cmd_local_shortcuts.rs
/// Proposito: resolver atajos capturados por la ventana con foco.
use super::AppState;
use crate::engine::input::actions as input_actions;
use crate::engine::input::profile_shortcuts::{self, ProfileAction};
use crate::engine::persist::config_io as config;
use crate::ipc::cmd_button_playback;
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
    code: Option<String>,
    pressed: Option<bool>,
    repeat: Option<bool>,
    state: tauri::State<AppState>,
) -> Result<LocalShortcutResult, String> {
    let source = format!("local:{}", code.unwrap_or_else(|| key.clone()));
    if pressed == Some(false) {
        state
            .mic
            .send(crate::engine::mic::Command::Input(source, false))?;
        return Ok(result(false, false));
    }
    let action = {
        let cfg = state.config.lock().unwrap();
        resolve_local_action(&cfg, &key)?
    };

    match action {
        LocalShortcutAction::None => Ok(result(false, false)),
        LocalShortcutAction::Profile(action) => {
            if action == ProfileAction::Mic && repeat == Some(true) {
                return Ok(result(true, false));
            }
            Ok(result(
                true,
                profile_shortcuts::execute(&state, action, &source)?,
            ))
        }
        LocalShortcutAction::SetPaleta(id) => {
            {
                let mut cfg = state.config.lock().unwrap();
                if input_actions::activate_paleta(&mut cfg, &id)? {
                    config::save_config(&cfg)?;
                }
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LocalShortcutAction {
    None,
    Profile(ProfileAction),
    SetPaleta(String),
    PlayButton(String),
}

pub(crate) fn resolve_local_action(
    cfg: &AppConfig,
    key: &str,
) -> Result<LocalShortcutAction, String> {
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    if profile.audio.global_keys {
        return Ok(LocalShortcutAction::None);
    }

    if let Some(action) = profile_shortcuts::resolve(&profile.audio, |saved| same_key(saved, key)) {
        return Ok(LocalShortcutAction::Profile(action));
    }
    for paleta in &profile.paletas {
        if same_key(&paleta.shortcut, key) {
            return Ok(LocalShortcutAction::SetPaleta(paleta.id.clone()));
        }
    }
    if let Some(id) = input_actions::play_by_shortcut(cfg, key)? {
        return Ok(LocalShortcutAction::PlayButton(id));
    }
    Ok(LocalShortcutAction::None)
}

fn same_key(saved: &str, key: &str) -> bool {
    profile_shortcuts::same_key(saved, key)
}

fn result(handled: bool, refresh: bool) -> LocalShortcutResult {
    LocalShortcutResult { handled, refresh }
}
