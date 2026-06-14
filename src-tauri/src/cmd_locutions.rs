/// Módulo: cmd_locutions.rs
/// Propósito: Comandos IPC del módulo de Locuciones Dinámicas (Fase 6).
/// La resolución de archivos vive en locutions.rs y la red en weather.rs.

use super::AppState;
use crate::config;
use crate::types::AppConfig;
use crate::{locutions, weather};

/// Guarda la configuración del módulo. `module_enabled` también controla el
/// interruptor maestro (el mismo que pregunta el asistente de primer arranque).
#[tauri::command]
pub fn set_locution_config(
    module_enabled:  bool,
    time_enabled:    bool,
    time_folder:     String,
    weather_enabled: bool,
    temp_folder:     String,
    hum_folder:      String,
    weather_city:    String,
    weather_lat:     f64,
    weather_lon:     f64,
    weather_unit:    String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.weather_module_enabled = module_enabled;
    let l = &mut cfg.locutions;
    l.time_enabled    = time_enabled;
    l.time_folder     = time_folder;
    l.weather_enabled = weather_enabled;
    l.temp_folder     = temp_folder;
    l.hum_folder      = hum_folder;
    l.weather_city    = weather_city;
    l.weather_lat     = weather_lat;
    l.weather_lon     = weather_lon;
    l.weather_unit    = weather_unit;
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

/// Abre el selector nativo de carpetas. Devuelve la ruta elegida.
#[tauri::command]
pub fn pick_folder() -> Result<String, String> {
    rfd::FileDialog::new()
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or("Operación cancelada.".to_string())
}

/// Busca ciudades en la API de geocodificación (para el autocompletado).
#[tauri::command]
pub fn search_city(query: String) -> Result<Vec<weather::CityResult>, String> {
    weather::search_city(&query)
}

/// Lee el clima actual (con caché). Si faltan coordenadas pero hay ciudad,
/// weather_now la geocodifica y la persiste automáticamente.
#[tauri::command]
pub fn get_weather_now(force: Option<bool>, state: tauri::State<AppState>) -> Result<weather::WeatherNow, String> {
    weather::weather_now(&state.config, force.unwrap_or(false))
}

/// Anuncia la hora actual: HRS{hh}+MIN{mm} (o HRS{hh}_O en punto) en secuencia.
/// `folder`: carpeta propia del botón (estilo LFA); si está vacía se usa la
/// global de Ajustes (que requiere el bloque de hora activo).
#[tauri::command]
pub fn play_time_locution(
    id: String, volume: Option<f32>, folder: Option<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let folder = match folder.filter(|f| !f.is_empty()) {
        Some(f) => f,
        None => {
            let cfg = state.config.lock().unwrap();
            if !(cfg.weather_module_enabled && cfg.locutions.time_enabled) {
                return Err("time_disabled".to_string());
            }
            cfg.locutions.time_folder.clone()
        }
    };
    let files = locutions::resolve_time_files(&folder)?;
    state.audio.lock().unwrap().play_sequence(id, files, volume.unwrap_or(1.0))
}

/// Anuncia temperatura o humedad actual. `kind`: "temperature" | "humidity".
/// `folder`: carpeta propia del botón (estilo LFA); si está vacía se usa la global.
#[tauri::command]
pub fn play_climate_locution(
    id: String, kind: String, volume: Option<f32>, folder: Option<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let folder = match folder.filter(|f| !f.is_empty()) {
        Some(f) => f,
        None => {
            let cfg = state.config.lock().unwrap();
            let l = &cfg.locutions;
            if kind == "humidity" { l.hum_folder.clone() } else { l.temp_folder.clone() }
        }
    };
    let now   = weather::weather_now(&state.config, false)?;
    let value = if kind == "humidity" { now.hum } else { now.temp };
    let file  = locutions::resolve_climate_file(&folder, &kind, value)?;
    state.audio.lock().unwrap().play_sequence(id, vec![file], volume.unwrap_or(1.0))
}
