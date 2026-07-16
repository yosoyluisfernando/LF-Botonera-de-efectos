/// Modulo: cmd_master_volume.rs
/// Proposito: comandos IPC y reglas de configuracion del volumen master.
///
/// El master **es el fader del bus Programa** de la consola: subirlo o bajarlo
/// es mover ese fader, y el vumetro de la barra inferior es su medidor. Por eso
/// se le pide a la consola y no al motor de efectos, que solo es uno de los
/// buses que suman en el programa.
use super::AppState;
use crate::engine::console::BusId;
use crate::engine::persist::config_io as config;
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
    let volume = state.console.fader(BusId::Programa);
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
    state.console.set_fader(BusId::Programa, volume);
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
        state.console.fader(BusId::Programa)
    };
    let volume = clamp_volume(volume, boost);
    audio.master_volume = volume;
    config::save_config(&cfg)?;
    drop(cfg);
    state.console.set_fader(BusId::Programa, volume);
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
    cfg.active_audio()
        .ok_or("active_profile_not_found".to_string())
}

fn active_audio_mut(cfg: &mut AppConfig) -> Result<&mut AudioConfig, String> {
    cfg.active_profile_mut()
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
