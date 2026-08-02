use super::midi_message::MidiInputEvent;
use crate::core::AppState;
use crate::engine::input::actions as input_actions;
use crate::engine::persist::config_io;
use crate::ipc::cmd_button_playback;
use crate::model::{AppConfig, MidiBinding};
use tauri::{AppHandle, Emitter, Manager};

pub fn dispatch(app: &AppHandle, event: &MidiInputEvent) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let action = {
        let cfg = state.config.lock().unwrap();
        resolve(&cfg, event)?
    };
    execute(&state, app, action)
}

enum MidiAction {
    None,
    StopAll,
    Cycle(i32),
    SetPaleta(String),
    PlayButton(String),
}

fn resolve(cfg: &AppConfig, event: &MidiInputEvent) -> Result<MidiAction, String> {
    if !cfg.midi.enabled {
        return Ok(MidiAction::None);
    }
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    if matches_binding(&profile.audio.midi_stop, event) {
        return Ok(MidiAction::StopAll);
    }
    if matches_binding(&profile.audio.midi_next, event) {
        return Ok(MidiAction::Cycle(1));
    }
    if matches_binding(&profile.audio.midi_prev, event) {
        return Ok(MidiAction::Cycle(-1));
    }
    for paleta in &profile.paletas {
        if matches_binding(&paleta.midi, event) {
            return Ok(MidiAction::SetPaleta(paleta.id.clone()));
        }
    }
    let fixed = if cfg.fixed_panel.scope == "profile" {
        &profile.fixed_buttons
    } else {
        &cfg.fixed_panel.global_buttons
    };
    for button in fixed {
        if matches_binding(&button.midi, event) {
            return Ok(MidiAction::PlayButton(button.id.clone()));
        }
    }
    let active = profile
        .paletas
        .iter()
        .find(|paleta| paleta.id == profile.active_paleta_id);
    if let Some(button) = active
        .into_iter()
        .flat_map(|paleta| paleta.botones.iter())
        .find(|button| matches_binding(&button.midi, event))
    {
        return Ok(MidiAction::PlayButton(button.id.clone()));
    }
    Ok(MidiAction::None)
}

fn execute(state: &AppState, app: &AppHandle, action: MidiAction) -> Result<bool, String> {
    match action {
        MidiAction::None => Ok(false),
        MidiAction::StopAll => {
            state.audio.lock().unwrap().stop_all();
            Ok(true)
        }
        MidiAction::Cycle(offset) => {
            {
                let mut cfg = state.config.lock().unwrap();
                input_actions::cycle_paleta(&mut cfg, offset)?;
                config_io::save_config(&cfg)?;
            }
            crate::engine::cache::warm::warm_visible_tab(state);
            app.emit("global-shortcut-refresh", ())
                .map_err(|error| error.to_string())?;
            Ok(true)
        }
        MidiAction::SetPaleta(id) => {
            {
                let mut cfg = state.config.lock().unwrap();
                if input_actions::activate_paleta(&mut cfg, &id)? {
                    config_io::save_config(&cfg)?;
                }
            }
            crate::engine::cache::warm::warm_visible_tab(state);
            app.emit("global-shortcut-refresh", ())
                .map_err(|error| error.to_string())?;
            Ok(true)
        }
        MidiAction::PlayButton(id) => {
            cmd_button_playback::play_button_id(state, &id)?;
            Ok(true)
        }
    }
}

pub fn matches_binding(binding: &MidiBinding, event: &MidiInputEvent) -> bool {
    !binding.is_empty()
        && binding.device_id == event.device_id
        && binding.message == event.message
        && binding.channel == event.channel
        && binding.number == event.number
}
