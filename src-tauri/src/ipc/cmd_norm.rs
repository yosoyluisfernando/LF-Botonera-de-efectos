/// Modulo: cmd_norm.rs
/// Proposito: comandos IPC de normalizacion, deteccion de cue y fundidos.
use super::AppState;
use crate::engine::cache::waveform_disk::{self, WaveformDiskStats};
use crate::engine::persist::config_io as config;
use crate::model::fade::FadeConfig;
use crate::model::norm::{CueDetectConfig, NormConfig};
use crate::model::waveform_cache::WaveformCacheConfig;

#[tauri::command]
pub fn set_norm_config(config_in: NormConfig, state: tauri::State<AppState>) -> Result<(), String> {
    if !matches!(config_in.mode.as_str(), "lufs" | "peak") {
        return Err("invalid_norm_mode".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    cfg.norm = config_in;
    config::save_config(&cfg)
}

#[tauri::command]
pub fn set_fade_config(config_in: FadeConfig, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.fade = config_in;
    config::save_config(&cfg)
}

#[tauri::command]
pub fn set_cue_detect_config(
    config_in: CueDetectConfig,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if config_in.enabled && !config_in.detect_start && !config_in.detect_end {
        return Err("invalid_cue_detect_config".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    cfg.cue_detect = config_in;
    config::save_config(&cfg)
}

#[tauri::command]
pub fn get_waveform_cache_stats(state: tauri::State<AppState>) -> WaveformDiskStats {
    let cfg = state.config.lock().unwrap().waveform_cache.clone();
    waveform_disk::stats(&cfg)
}

#[tauri::command]
pub fn clear_waveform_cache(state: tauri::State<AppState>) -> Result<WaveformDiskStats, String> {
    waveform_disk::clear()?;
    let cfg = state.config.lock().unwrap().waveform_cache.clone();
    Ok(waveform_disk::stats(&cfg))
}

#[tauri::command]
pub fn set_waveform_cache_config(
    config_in: WaveformCacheConfig,
    state: tauri::State<AppState>,
) -> Result<WaveformDiskStats, String> {
    let next = config_in.sanitized();
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.waveform_cache = next.clone();
        config::save_config(&cfg)?;
    }
    waveform_disk::cleanup(&next)
}

#[tauri::command]
pub fn mark_norm_prompted(state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.norm_prompted = true;
    config::save_config(&cfg)
}
