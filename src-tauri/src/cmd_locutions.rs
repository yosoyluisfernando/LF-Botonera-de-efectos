/// Módulo: cmd_locutions.rs
/// Propósito: Comandos IPC del módulo de Locuciones Dinámicas (Fase 6).
/// La resolución de archivos vive en locutions.rs y la red en weather.rs.
use super::AppState;
use crate::config;
use crate::types::AppConfig;
use crate::{locution_playback, weather};
use serde::Serialize;

#[derive(Serialize)]
pub struct PickedFolder {
    pub path: String,
    pub name: String,
}

/// Guarda la configuración del módulo. `module_enabled` también controla el
/// interruptor maestro (el mismo que pregunta el asistente de primer arranque).
#[tauri::command]
pub fn set_locution_config(
    module_enabled: bool,
    time_enabled: bool,
    time_folder: String,
    weather_enabled: bool,
    temp_folder: String,
    hum_folder: String,
    weather_city: String,
    weather_lat: f64,
    weather_lon: f64,
    weather_unit: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let normalized = normalize_locution_activation(
        module_enabled,
        time_enabled,
        &time_folder,
        weather_enabled,
        &temp_folder,
        &hum_folder,
    );
    let mut cfg = state.config.lock().unwrap();
    cfg.weather_module_enabled = normalized.module_enabled;
    let l = &mut cfg.locutions;
    l.time_enabled = normalized.time_enabled;
    l.time_folder = time_folder;
    l.weather_enabled = normalized.weather_enabled;
    l.temp_folder = temp_folder;
    l.hum_folder = hum_folder;
    l.weather_city = weather_city;
    l.weather_lat = weather_lat;
    l.weather_lon = weather_lon;
    l.weather_unit = weather_unit;
    config::save_config(&cfg)?;
    if let Some(error_key) = normalized.error_key {
        return Err(error_key.to_string());
    }
    Ok(cfg.clone())
}

/// Abre el selector nativo de carpetas y devuelve ruta mas nombre visible.
#[tauri::command]
pub fn pick_named_folder() -> Result<PickedFolder, String> {
    let path = rfd::FileDialog::new()
        .pick_folder()
        .ok_or("OperaciÃ³n cancelada.".to_string())?;
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    Ok(PickedFolder {
        path: path.to_string_lossy().to_string(),
        name,
    })
}

struct LocutionActivation {
    module_enabled: bool,
    time_enabled: bool,
    weather_enabled: bool,
    error_key: Option<&'static str>,
}

/// Conserva siempre las rutas editadas, pero no deja activo un bloque que no
/// tiene sus carpetas obligatorias. Rust mantiene la autoridad de validacion.
fn normalize_locution_activation(
    module_enabled: bool,
    time_enabled: bool,
    time_folder: &str,
    weather_enabled: bool,
    temp_folder: &str,
    hum_folder: &str,
) -> LocutionActivation {
    let mut normalized = LocutionActivation {
        module_enabled,
        time_enabled,
        weather_enabled,
        error_key: None,
    };

    if !module_enabled {
        return normalized;
    }
    if time_enabled && time_folder.trim().is_empty() {
        normalized.time_enabled = false;
        normalized.error_key = Some("loc_missing_time_folder");
        return normalized;
    }
    if weather_enabled && temp_folder.trim().is_empty() {
        normalized.weather_enabled = false;
        normalized.error_key = Some("loc_missing_temp_folder");
        return normalized;
    }
    if weather_enabled && hum_folder.trim().is_empty() {
        normalized.weather_enabled = false;
        normalized.error_key = Some("loc_missing_hum_folder");
    }
    normalized
}

/// Busca ciudades en la API de geocodificación (para el autocompletado).
#[tauri::command]
pub fn search_city(query: String) -> Result<Vec<weather::CityResult>, String> {
    weather::search_city(&query)
}

/// Lee el clima actual (con caché). Si faltan coordenadas pero hay ciudad,
/// weather_now la geocodifica y la persiste automáticamente.
#[tauri::command]
pub fn get_weather_now(
    force: Option<bool>,
    state: tauri::State<AppState>,
) -> Result<weather::WeatherNow, String> {
    weather::weather_now(&state.config, force.unwrap_or(false))
}

/// Anuncia la hora actual: HRS{hh}+MIN{mm} (o HRS{hh}_O en punto) en secuencia.
/// `folder`: carpeta propia del botón (estilo LFA); si está vacía se usa la
/// global de Ajustes (que requiere el bloque de hora activo).
#[tauri::command]
pub fn play_time_locution(
    id: String,
    volume: Option<f32>,
    folder: Option<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let cfg = state.config.lock().unwrap().clone();
    locution_playback::play_time(&state, &cfg, id, volume.unwrap_or(1.0), folder.as_deref())
}

/// Anuncia temperatura o humedad actual. `kind`: "temperature" | "humidity".
/// `folder`: carpeta propia del botón (estilo LFA); si está vacía se usa la global.
#[tauri::command]
pub fn play_climate_locution(
    id: String,
    kind: String,
    volume: Option<f32>,
    folder: Option<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let cfg = state.config.lock().unwrap().clone();
    locution_playback::play_climate(
        &state,
        &cfg,
        id,
        &kind,
        volume.unwrap_or(1.0),
        folder.as_deref(),
    )
}
