use super::AppState;
use crate::domain::playback::mode::PlaybackMode;
use crate::engine::audio::button::PlaybackGroup;
use crate::engine::persist::config_io;
use serde::Serialize;

#[derive(Serialize)]
pub struct FixedPlaybackState {
    pub mode: String,
    pub solo: bool,
}

#[tauri::command]
pub fn get_fixed_playback_state(state: tauri::State<AppState>) -> FixedPlaybackState {
    let cfg = state.config.lock().unwrap();
    FixedPlaybackState {
        mode: PlaybackMode::from_config(&cfg.fixed_panel.playback_mode)
            .as_str()
            .into(),
        solo: cfg.fixed_panel.solo_mode,
    }
}

#[tauri::command]
pub fn set_fixed_playback_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mode = PlaybackMode::parse(&mode)?;
    let mut cfg = state.config.lock().unwrap();
    cfg.fixed_panel.playback_mode = mode.as_str().into();
    config_io::save_config(&cfg)
}

#[tauri::command]
pub fn set_fixed_solo_mode(enabled: bool, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.fixed_panel.solo_mode = enabled;
    config_io::save_config(&cfg)
}

#[tauri::command]
pub fn stop_fixed_audio(state: tauri::State<AppState>) {
    state
        .audio
        .lock()
        .unwrap()
        .stop_group_fade(PlaybackGroup::Fixed);
}
