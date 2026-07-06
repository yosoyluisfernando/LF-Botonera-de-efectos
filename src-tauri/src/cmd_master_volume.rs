/// Modulo: cmd_master_volume.rs
/// Proposito: comandos IPC y reglas de configuracion del volumen master.
use super::AppState;
use crate::config;
use crate::model::{AppConfig, AudioConfig};
use serde::Serialize;

#[derive(Serialize)]
pub struct MasterVolumeState {
    pub volume: f32,
    pub remember: bool,
    pub boost: bool,
    pub max: f32,
}

#[tauri::command]
pub fn get_master_volume_state(state: tauri::State<AppState>) -> Result<MasterVolumeState, String> {
    let cfg = state.config.lock().unwrap();
    let audio = active_audio(&cfg)?;
    let volume = state.audio.lock().unwrap().master_volume();
    Ok(MasterVolumeState {
        volume,
        remember: audio.master_volume_remember,
        boost: audio.master_volume_boost,
        max: max_volume(audio.master_volume_boost),
    })
}

#[tauri::command]
pub fn set_master_volume(
    volume: f32,
    state: tauri::State<AppState>,
) -> Result<MasterVolumeState, String> {
    let mut cfg = state.config.lock().unwrap();
    let audio = active_audio_mut(&mut cfg)?;
    let volume = clamp_volume(volume, audio.master_volume_boost);
    audio.master_volume = volume;
    let remember = audio.master_volume_remember;
    let boost = audio.master_volume_boost;
    if remember {
        config::save_config(&cfg)?;
    }
    drop(cfg);
    state.audio.lock().unwrap().set_master_volume(volume);
    Ok(MasterVolumeState {
        volume,
        remember,
        boost,
        max: max_volume(boost),
    })
}

#[tauri::command]
pub fn set_master_volume_options(
    remember: bool,
    boost: bool,
    state: tauri::State<AppState>,
) -> Result<MasterVolumeState, String> {
    let mut cfg = state.config.lock().unwrap();
    let audio = active_audio_mut(&mut cfg)?;
    audio.master_volume_remember = remember;
    audio.master_volume_boost = boost;
    audio.master_volume = clamp_volume(audio.master_volume, boost);
    let volume = if remember {
        audio.master_volume
    } else {
        state.audio.lock().unwrap().master_volume()
    };
    let volume = clamp_volume(volume, boost);
    audio.master_volume = volume;
    config::save_config(&cfg)?;
    drop(cfg);
    state.audio.lock().unwrap().set_master_volume(volume);
    Ok(MasterVolumeState {
        volume,
        remember,
        boost,
        max: max_volume(boost),
    })
}

pub fn startup_volume(cfg: &AppConfig) -> f32 {
    active_audio(cfg)
        .map(|audio| {
            if audio.master_volume_remember {
                clamp_volume(audio.master_volume, audio.master_volume_boost)
            } else {
                1.0
            }
        })
        .unwrap_or(1.0)
}

fn active_audio(cfg: &AppConfig) -> Result<&AudioConfig, String> {
    cfg.profiles
        .iter()
        .find(|profile| profile.id == cfg.active_profile_id)
        .map(|profile| &profile.audio)
        .ok_or("active_profile_not_found".to_string())
}

fn active_audio_mut(cfg: &mut AppConfig) -> Result<&mut AudioConfig, String> {
    cfg.profiles
        .iter_mut()
        .find(|profile| profile.id == cfg.active_profile_id)
        .map(|profile| &mut profile.audio)
        .ok_or("active_profile_not_found".to_string())
}

fn clamp_volume(volume: f32, boost: bool) -> f32 {
    volume.clamp(0.0, max_volume(boost))
}

fn max_volume(boost: bool) -> f32 {
    if boost {
        1.5
    } else {
        1.0
    }
}
