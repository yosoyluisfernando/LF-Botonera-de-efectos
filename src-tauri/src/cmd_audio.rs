/// Módulo: cmd_audio.rs
/// Propósito: Comandos IPC relacionados con el motor de audio.

use super::AppState;
use crate::config;

#[tauri::command]
pub fn get_audio_devices(state: tauri::State<AppState>) -> Vec<String> {
    state.audio.lock().unwrap().get_available_devices()
}

#[tauri::command]
pub fn set_audio_device(
    device_name: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    state.audio.lock().unwrap().set_device(&device_name)?;
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    if let Some(p) = cfg.profiles.iter_mut().find(|p| p.id == pid) {
        p.audio.out_main = device_name;
    }
    config::save_config(&cfg)
}

/// Reproduce un archivo. Aplica el modo de reproducción global del perfil activo:
/// si el modo es distinto de "normal", sobreescribe los flags del botón individual.
#[tauri::command]
pub fn play_audio(
    id:         String,
    path:       String,
    volume:     f32,
    duration:   Option<f64>,
    loop_mode:  Option<bool>,
    stop_other: Option<bool>,
    overlap:    Option<bool>,
    restart:    Option<bool>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mode = {
        let cfg = state.config.lock().unwrap();
        let pid = cfg.active_profile_id.clone();
        cfg.profiles.iter().find(|p| p.id == pid)
            .map(|p| p.audio.playback_mode.clone())
            .unwrap_or_else(|| "normal".to_string())
    };
    let (floop, fstop, foverlap, frestart) = match mode.as_str() {
        "loop"        => (true,  false, false, false),
        "overlap"     => (false, false, true,  false),
        "restart"     => (false, false, false, true),
        "stop_others" => (false, true,  false, false),
        _             => (loop_mode.unwrap_or(false), stop_other.unwrap_or(false),
                          overlap.unwrap_or(false), restart.unwrap_or(false)),
    };
    state.audio.lock().unwrap().play_file(
        id, &path, volume, duration.unwrap_or(0.0),
        floop, fstop, foverlap, frestart,
    )
}

#[tauri::command]
pub fn stop_audio(id: String, state: tauri::State<AppState>) {
    state.audio.lock().unwrap().stop(&id);
}

#[tauri::command]
pub fn stop_all_audio(state: tauri::State<AppState>) {
    state.audio.lock().unwrap().stop_all();
}

/// Ajusta en vivo el volumen de un sonido activo (usado por la pre-escucha).
#[tauri::command]
pub fn set_audio_volume(id: String, volume: f32, state: tauri::State<AppState>) {
    state.audio.lock().unwrap().set_volume(&id, volume);
}

/// Sonda la duración de un archivo leyendo sus propiedades (lofty), sin
/// decodificarlo. Funciona con MP3, WAV, OGG, FLAC, M4A. Devuelve -1 si falla.
/// (rodio::Decoder::total_duration devuelve None en la mayoría de MP3,
/// por eso se usa lofty como fuente de verdad.)
pub fn probe_duration_secs(path: &str) -> f64 {
    use lofty::file::AudioFile;
    lofty::read_from_path(path)
        .map(|f| f.properties().duration().as_secs_f64())
        .unwrap_or(-1.0)
}

#[tauri::command]
pub fn probe_duration(path: String) -> f64 {
    probe_duration_secs(&path)
}
