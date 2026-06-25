/// Módulo: cmd_locutions.rs
/// Propósito: Comandos IPC del módulo de Locuciones Dinámicas (Fase 6).
/// La resolución de archivos vive en locutions.rs y la red en weather.rs.
use super::AppState;
use crate::config;
use crate::types::AppConfig;
use crate::{geocode, locution_playback, weather};
use serde::Serialize;

#[derive(Serialize)]
pub struct PickedFolder {
    pub path: String,
    pub name: String,
}

/// Guarda la configuración del módulo. `module_enabled` también controla el
/// interruptor maestro (el mismo que pregunta el asistente de primer arranque).
/// La UI nunca envía coordenadas: Rust las resuelve a partir de la ciudad y las
/// persiste él mismo (fuente única de verdad).
#[tauri::command]
pub fn set_locution_config(
    module_enabled: bool,
    time_enabled: bool,
    time_folder: String,
    weather_enabled: bool,
    temp_folder: String,
    hum_folder: String,
    weather_city: String,
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
    // Si cambia la ciudad, invalidar las coordenadas: weather_now las resolverá
    // y persistirá de forma perezosa (y sin red no se bloquea el guardado).
    if l.weather_city != weather_city {
        l.weather_lat = 0.0;
        l.weather_lon = 0.0;
    }
    l.weather_city = weather_city;
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

/// Conserva siempre las rutas editadas, pero no deja activo un bloque sin sus
/// carpetas: hora exige su carpeta y clima exige al menos una (temperatura o
/// humedad). Rust mantiene la autoridad de validacion.
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
    // El clima necesita AL MENOS una locución (temperatura o humedad); el
    // usuario puede configurar solo una de las dos.
    if weather_enabled && temp_folder.trim().is_empty() && hum_folder.trim().is_empty() {
        normalized.weather_enabled = false;
        normalized.error_key = Some("loc_missing_climate_folder");
    }
    normalized
}

/// Busca ciudades en la API de geocodificación (para el autocompletado).
#[tauri::command]
pub fn search_city(query: String) -> Result<Vec<geocode::CityResult>, String> {
    geocode::search_city(&query)
}

/// Comprueba el clima de una ciudad escrita en el panel, sin tocar la
/// configuración ni exigir carpetas. Respaldo del botón "Comprobar".
#[tauri::command]
pub fn preview_weather(
    city: String,
    unit: String,
) -> Result<weather::WeatherPreview, String> {
    weather::preview_weather(&city, &unit)
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
