//! Reglas de control OBS; sin red, Tauri ni acceso a disco.
pub mod actions;
mod button_state;
mod filter_record;
#[cfg(test)]
mod filter_record_tests;
mod resource_actions;
#[cfg(test)]
mod resource_format_tests;
mod resource_states;
#[cfg(test)]
mod resource_tests;
#[cfg(test)]
mod tests;
pub mod view;
use crate::model::{obs::ObsConfig, AppConfig, PaletaData};

pub fn connection_changed(previous: &ObsConfig, next: &ObsConfig) -> bool {
    previous.enabled != next.enabled
        || previous.host != next.host
        || previous.port != next.port
        || previous.reconnect != next.reconnect
}

pub fn validate_config(cfg: &ObsConfig) -> Result<(), String> {
    if cfg.port == 0
        || cfg.host.is_empty()
        || cfg.host.len() > 253
        || cfg
            .host
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '/' | '\\' | '@' | '?' | '#'))
    {
        return Err("obs_invalid_connection".into());
    }
    Ok(())
}
pub fn audio_paleta(p: &PaletaData) -> Result<(), String> {
    if p.obs {
        Err("obs_separate_palettes".into())
    } else {
        Ok(())
    }
}
pub fn palette(cfg: &AppConfig, id: &str) -> Result<(), String> {
    let p = cfg
        .active_profile()
        .and_then(|p| p.paletas.iter().find(|p| p.id == id))
        .ok_or("tab_not_found")?;
    audio_paleta(p)
}
pub fn compatible(from: &PaletaData, to: &PaletaData) -> Result<(), String> {
    if from.obs != to.obs {
        Err("obs_separate_palettes".into())
    } else {
        Ok(())
    }
}

/// El archivo importado nunca puede introducir controles en destinos de audio.
pub fn validate_paleta(p: &PaletaData) -> Result<(), String> {
    for b in &p.botones {
        if p.obs != (b.type_field == "obs") {
            return Err("obs_separate_palettes".into());
        }
        if p.obs {
            actions::validate(b.obs.as_ref().ok_or("obs_invalid_action")?)?;
        }
    }
    Ok(())
}
pub fn validate_profile(p: &crate::model::ProfileData) -> Result<(), String> {
    for paleta in &p.paletas {
        validate_paleta(paleta)?;
    }
    if p.fixed_buttons.iter().any(|b| b.type_field == "obs") {
        return Err("obs_separate_palettes".into());
    }
    Ok(())
}
