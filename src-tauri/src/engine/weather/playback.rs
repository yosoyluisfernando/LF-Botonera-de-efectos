use crate::core::AppState;
use crate::engine::audio::button::PlaybackGroup;
/// Modulo: locution_playback.rs
/// Proposito: reproducir locuciones de hora y clima por una ruta unica.
use crate::engine::audio::formats::probe_duration_secs;
use crate::engine::weather::client as weather;
use crate::engine::weather::resolver as locutions;
use crate::model::AppConfig;

/// Reproduce la locucion de hora usando carpeta propia o configuracion global.
pub fn play_time(
    state: &AppState,
    cfg: &AppConfig,
    id: String,
    volume: f32,
    folder: Option<&str>,
    group: PlaybackGroup,
) -> Result<(), String> {
    let folder = resolve_time_folder(cfg, folder)?;
    let files = locutions::resolve_time_files(&folder)?;
    let duration = total_duration(&files);
    state
        .audio
        .lock()
        .unwrap()
        .play_sequence(id, files, volume, duration, group)
}

/// Reproduce locucion de temperatura o humedad usando el clima actual.
pub fn play_climate(
    state: &AppState,
    cfg: &AppConfig,
    id: String,
    kind: &str,
    volume: f32,
    folder: Option<&str>,
    group: PlaybackGroup,
) -> Result<(), String> {
    let folder = resolve_climate_folder(cfg, kind, folder)?;
    let now = weather::weather_now(&state.config, false)?;
    let value = if kind == "humidity" {
        now.hum
    } else {
        now.temp
    };
    let file = locutions::resolve_climate_file(&folder, kind, value)?;
    let duration = total_duration(std::slice::from_ref(&file));
    state
        .audio
        .lock()
        .unwrap()
        .play_sequence(id, vec![file], volume, duration, group)
}

fn total_duration(paths: &[String]) -> f64 {
    paths
        .iter()
        .map(|path| probe_duration_secs(path))
        .filter(|duration| *duration > 0.0)
        .sum()
}

/// Carpeta de la locucion horaria: la propia de la fila si la trae; si no, la de
/// Ajustes (y solo si el modulo esta activo). La comparte el reproductor
/// auxiliar, para que una locucion se ubique igual venga de un boton o de la cola.
pub(crate) fn resolve_time_folder(cfg: &AppConfig, folder: Option<&str>) -> Result<String, String> {
    if let Some(folder) = filled(folder) {
        return Ok(folder.to_string());
    }
    if !(cfg.weather_module_enabled && cfg.locutions.time_enabled) {
        return Err("time_disabled".to_string());
    }
    Ok(cfg.locutions.time_folder.clone())
}

/// Igual que `resolve_time_folder`, para temperatura y humedad.
pub(crate) fn resolve_climate_folder(
    cfg: &AppConfig,
    kind: &str,
    folder: Option<&str>,
) -> Result<String, String> {
    if !matches!(kind, "temperature" | "humidity") {
        return Err("invalid_locution_kind".to_string());
    }
    if let Some(folder) = filled(folder) {
        return Ok(folder.to_string());
    }
    if !(cfg.weather_module_enabled && cfg.locutions.weather_enabled) {
        return Err("weather_disabled".to_string());
    }
    if kind == "humidity" {
        Ok(cfg.locutions.hum_folder.clone())
    } else {
        Ok(cfg.locutions.temp_folder.clone())
    }
}

fn filled(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.trim().is_empty())
}
