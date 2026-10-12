//! Acciones de teclado del perfil, compartidas por captura local y global.
use crate::core::AppState;
use crate::engine::persist::config_io;
use crate::model::{AppConfig, AudioConfig};
use tauri_plugin_global_shortcut::Shortcut;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileAction {
    StopAll,
    Cycle(i32),
    PlayerPlay,
    PlayerStop,
    PlayerNext,
    Mic,
}

pub fn bindings(audio: &AudioConfig) -> [(ProfileAction, &str); 7] {
    [
        (ProfileAction::StopAll, &audio.key_stop),
        (ProfileAction::Cycle(1), &audio.key_next),
        (ProfileAction::Cycle(-1), &audio.key_prev),
        (ProfileAction::PlayerPlay, &audio.key_player_play),
        (ProfileAction::PlayerStop, &audio.key_player_stop),
        (ProfileAction::PlayerNext, &audio.key_player_next),
        (ProfileAction::Mic, &audio.key_mic),
    ]
}

pub fn resolve(audio: &AudioConfig, matches: impl Fn(&str) -> bool) -> Option<ProfileAction> {
    bindings(audio)
        .into_iter()
        .find_map(|(action, key)| (!key.trim().is_empty() && matches(key)).then_some(action))
}

/// Devuelve si la accion requiere refrescar las pestañas. El transporte del
/// reproductor usa los mismos metodos que sus botones y pinta por player-tick.
pub fn execute(state: &AppState, action: ProfileAction, source: &str) -> Result<bool, String> {
    match action {
        ProfileAction::StopAll => state.audio.lock().unwrap().stop_all(),
        ProfileAction::Cycle(offset) => {
            let mut cfg = state.config.lock().unwrap();
            super::actions::cycle_paleta(&mut cfg, offset)?;
            config_io::save_config(&cfg)?;
            drop(cfg);
            crate::engine::cache::warm::warm_visible_tab(state);
            return Ok(true);
        }
        ProfileAction::PlayerPlay => state.player.lock().unwrap().resume(),
        ProfileAction::PlayerStop => state.player.lock().unwrap().stop(),
        ProfileAction::PlayerNext => state.player.lock().unwrap().next(),
        ProfileAction::Mic => state
            .mic
            .send(crate::engine::mic::Command::Input(source.into(), true))?,
    }
    Ok(false)
}

pub fn parse(key: &str) -> Result<Shortcut, String> {
    key.trim()
        .replace("Ctrl+", "Control+")
        .replace("Espacio", "Space")
        .parse::<Shortcut>()
        .map_err(|_| "invalid_keyboard_shortcut".into())
}

pub fn same_key(a: &str, b: &str) -> bool {
    match (parse(a), parse(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => super::actions::same_key(a, b),
    }
}

/// Valida el conjunto propuesto antes de modificar configuracion o registro.
/// Permite intercambiar teclas entre acciones en una unica operacion.
pub fn validate(cfg: &AppConfig, audio: &AudioConfig) -> Result<(), String> {
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    let mut used = Vec::new();
    for (_, key) in bindings(audio) {
        if key.trim().is_empty() {
            continue;
        }
        if super::rules::is_reserved_system_key(key) {
            return Err("reserved_system_shortcut".into());
        }
        let parsed = parse(key)?;
        if used.contains(&parsed) {
            return Err("duplicate_keyboard_shortcut".into());
        }
        used.push(parsed);
        let buttons = profile
            .paletas
            .iter()
            .flat_map(|tab| &tab.botones)
            .chain(&profile.fixed_buttons)
            .chain(&cfg.fixed_panel.global_buttons);
        if profile
            .paletas
            .iter()
            .any(|tab| same_key(&tab.shortcut, key))
            || buttons
                .into_iter()
                .any(|button| same_key(&button.shortcut, key))
        {
            return Err("keyboard_shortcut_in_use".into());
        }
    }
    Ok(())
}
