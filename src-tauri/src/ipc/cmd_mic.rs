use crate::{
    core::AppState,
    engine::{
        mic::{Command, Snapshot},
        persist::config_io,
    },
    model::mic::MicConfig,
};

#[tauri::command]
pub fn get_mic(state: tauri::State<AppState>) -> (MicConfig, Snapshot) {
    (
        state.config.lock().unwrap().mic.clone(),
        state.mic.snapshot(),
    )
}

#[tauri::command]
pub async fn mic_devices() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(crate::engine::mic::capture::devices)
        .await
        .map_err(|_| "mic.input_error".to_string())?
}

#[tauri::command]
pub fn set_mic_config(config: MicConfig, state: tauri::State<AppState>) -> Result<(), String> {
    config.validate()?;
    let mut cfg = state.config.lock().unwrap();
    let mut next = cfg.clone();
    next.mic = config;
    config_io::save_config(&next)?;
    *cfg = next;
    Ok(())
}

#[tauri::command]
pub fn mic_preview(input: Option<String>, state: tauri::State<AppState>) -> Result<(), String> {
    state.mic.send(Command::Preview(input))
}

#[tauri::command]
pub fn mic_pointer(pressed: bool, state: tauri::State<AppState>) -> Result<(), String> {
    state.mic.send(Command::Input("pointer".into(), pressed))
}

#[tauri::command]
pub fn mic_release_local(state: tauri::State<AppState>) -> Result<(), String> {
    state.mic.send(Command::ReleaseLocal)
}
