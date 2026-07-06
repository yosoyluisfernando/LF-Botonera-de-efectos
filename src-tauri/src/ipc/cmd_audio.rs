/// Modulo: cmd_audio.rs
/// Proposito: comandos IPC relacionados con el motor de audio.
use super::AppState;
use crate::engine::audio::device as audio_device;
use crate::engine::audio::formats::validate_audio_file;
use crate::engine::persist::config_io as config;
use crate::model::fade::FadeConfig;
use serde::Serialize;

#[derive(Serialize)]
pub struct AudioDeviceStatus {
    pub devices: Vec<String>,
    pub main: String,
    pub pre: String,
    pub main_available: bool,
    pub pre_available: bool,
}

#[tauri::command]
pub fn get_audio_devices(state: tauri::State<AppState>) -> Vec<String> {
    state.audio.lock().unwrap().get_available_devices()
}

#[tauri::command]
pub fn get_audio_device_status(state: tauri::State<AppState>) -> AudioDeviceStatus {
    let devices = state.audio.lock().unwrap().get_available_devices();
    let (main, pre) = configured_devices(&state);
    AudioDeviceStatus {
        main_available: device_is_available(&main),
        pre_available: pre_device_is_available(&pre, &main),
        devices,
        main,
        pre,
    }
}

#[tauri::command]
pub fn apply_configured_audio_devices(state: tauri::State<AppState>) -> Result<AudioDeviceStatus, String> {
    let status = get_audio_device_status(state.clone());
    if !status.main_available || !status.pre_available {
        return Ok(status);
    }
    state.audio.lock().unwrap().set_device(&status.main)?;
    let effective_pre = if status.pre.is_empty() || status.pre == status.main {
        ""
    } else {
        &status.pre
    };
    state.audio.lock().unwrap().set_pre_device(effective_pre)?;
    Ok(status)
}

#[tauri::command]
pub fn set_audio_device(device_name: String, state: tauri::State<AppState>) -> Result<(), String> {
    state.audio.lock().unwrap().set_device(&device_name)?;
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    if let Some(p) = cfg.profiles.iter_mut().find(|p| p.id == pid) {
        p.audio.out_main = device_name;
    }
    config::save_config(&cfg)
}

/// Reproduce un archivo directo, usado por pre-escucha. Los botones normales
/// deben pasar por play_button para aplicar reglas de boton y modo global.
/// Acepta cue/ganancia opcionales: la previa del editor los envia para sonar
/// igual que sonara el boton (por defecto por el id de pre-escucha).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn play_audio(
    id: String,
    path: String,
    volume: f32,
    duration: Option<f64>,
    loop_mode: Option<bool>,
    stop_other: Option<bool>,
    overlap: Option<bool>,
    restart: Option<bool>,
    cue_start_s: Option<f64>,
    cue_end_s: Option<f64>,
    gain_db: Option<f64>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    validate_audio_file(&path)?;
    // Precarga el archivo en segundo plano: así adelantar/atrasar (seek) en la
    // pre-escucha o la previa del editor es instantáneo (O(1) en RAM).
    state.audio.lock().unwrap().enqueue_preload(path.clone());
    let file_gain = gain_db.map(crate::model::track::db_to_linear).unwrap_or(1.0);
    // Pre-escucha y preview del editor no usan fade: el operador controla el monitor
    // manualmente y un fade involuntario sería confuso.
    state.audio.lock().unwrap().play_file(
        id,
        &path,
        volume,
        duration.unwrap_or(0.0),
        loop_mode.unwrap_or(false),
        stop_other.unwrap_or(false),
        overlap.unwrap_or(false),
        restart.unwrap_or(false),
        cue_start_s.unwrap_or(0.0),
        cue_end_s,
        file_gain,
        true, // pre-escucha/previa → bus PRE (con fallback al principal)
        &FadeConfig::default(),
    )
}

/// Fija el dispositivo de pre-escucha. Aplica el fallback (Regla 4, lógica en
/// Rust): si está vacío o coincide con la salida principal, se usa la principal.
#[tauri::command]
pub fn set_pre_device(device_name: String, state: tauri::State<AppState>) -> Result<(), String> {
    let out_main = {
        let mut cfg = state.config.lock().unwrap();
        let pid = cfg.active_profile_id.clone();
        let mut main = String::new();
        if let Some(p) = cfg.profiles.iter_mut().find(|p| p.id == pid) {
            p.audio.out_pre = device_name.clone();
            main = p.audio.out_main.clone();
        }
        config::save_config(&cfg)?;
        main
    };
    let effective = if device_name.is_empty() || device_name == out_main {
        String::new()
    } else {
        device_name
    };
    state.audio.lock().unwrap().set_pre_device(&effective)
}

#[tauri::command]
pub fn stop_audio(id: String, state: tauri::State<AppState>) {
    let fade_s = state.config.lock().unwrap().fade.fade_out_stop_s;
    let audio = state.audio.lock().unwrap();
    if fade_s > 0.0 { audio.stop_fade(&id) } else { audio.stop(&id) }
}

#[tauri::command]
pub fn stop_all_audio(state: tauri::State<AppState>) {
    let fade_s = state.config.lock().unwrap().fade.fade_out_stop_s;
    let audio = state.audio.lock().unwrap();
    if fade_s > 0.0 { audio.stop_all_fade() } else { audio.stop_all() }
}

/// Ajusta en vivo el volumen de un sonido activo (usado por la pre-escucha).
#[tauri::command]
pub fn set_audio_volume(id: String, volume: f32, state: tauri::State<AppState>) {
    state.audio.lock().unwrap().set_volume(&id, volume);
}

/// Sonda la duracion de un archivo leyendo sus propiedades (lofty), sin
/// decodificarlo. Funciona con MP3, WAV, OGG, FLAC, M4A. Devuelve -1 si falla.
/// (rodio::Decoder::total_duration devuelve None en la mayoria de MP3,
/// por eso se usa lofty como fuente de verdad.)
pub fn probe_duration_secs(path: &str) -> f64 {
    use lofty::file::AudioFile;
    lofty::read_from_path(path)
        .map(|f| f.properties().duration().as_secs_f64())
        .unwrap_or(-1.0)
}

fn configured_devices(state: &AppState) -> (String, String) {
    let cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let audio = cfg.profiles.iter().find(|p| p.id == pid).map(|p| &p.audio);
    (
        audio.map(|a| a.out_main.clone()).unwrap_or_else(|| "default".to_string()),
        audio.map(|a| a.out_pre.clone()).unwrap_or_default(),
    )
}

fn device_is_available(device_name: &str) -> bool {
    let device = if device_name.trim().is_empty() { "default" } else { device_name };
    audio_device::device_available(device)
}

fn pre_device_is_available(pre: &str, main: &str) -> bool {
    pre.trim().is_empty() || (pre == main && device_is_available(main)) || device_is_available(pre)
}
