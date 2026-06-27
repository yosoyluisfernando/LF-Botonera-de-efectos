/// Modulo: cmd_audio.rs
/// Proposito: comandos IPC relacionados con el motor de audio.
use super::AppState;
use crate::audio_formats::validate_audio_file;
use crate::config;

#[tauri::command]
pub fn get_audio_devices(state: tauri::State<AppState>) -> Vec<String> {
    state.audio.lock().unwrap().get_available_devices()
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
    let file_gain = gain_db.map(crate::types_track::db_to_linear).unwrap_or(1.0);
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
