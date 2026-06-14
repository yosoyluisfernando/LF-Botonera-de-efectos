/// Módulo: weather.rs
/// Propósito: Cliente de la API Open-Meteo (la misma que usa el LF
/// Automatizador): geocodificación de ciudades, lectura de temperatura /
/// humedad, caché en memoria y refresco automático cada 15 minutos (como el
/// LFA) que emite el evento "weather-updated" hacia la UI.

use crate::config;
use crate::types::AppConfig;
use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// TTL del caché: el clima no cambia tan rápido como para refrescar siempre.
const CACHE_TTL: Duration = Duration::from_secs(600);
/// Intervalo del refresco automático (15 min, igual que el LF Automatizador).
const AUTO_REFRESH: Duration = Duration::from_secs(900);

#[derive(Serialize, Clone, Copy)]
pub struct WeatherNow {
    pub temp: f64,
    pub hum:  f64,
}

#[derive(Serialize, Clone)]
pub struct CityResult {
    pub label: String,
    pub lat:   f64,
    pub lon:   f64,
}

static CACHE: Mutex<Option<(Instant, WeatherNow)>> = Mutex::new(None);

/// Busca ciudades por nombre (geocodificación, igual que el LFA).
pub fn search_city(query: &str) -> Result<Vec<CityResult>, String> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=5&language=es&format=json",
        urlencode(query)
    );
    let json: serde_json::Value = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call().map_err(|e| e.to_string())?
        .into_json().map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(results) = json["results"].as_array() {
        for r in results {
            let name    = r["name"].as_str().unwrap_or_default();
            let admin   = r["admin1"].as_str().map(|a| format!(", {}", a)).unwrap_or_default();
            let country = r["country_code"].as_str().unwrap_or_default();
            out.push(CityResult {
                label: format!("{}{}, {}", name, admin, country),
                lat:   r["latitude"].as_f64().unwrap_or(0.0),
                lon:   r["longitude"].as_f64().unwrap_or(0.0),
            });
        }
    }
    Ok(out)
}

/// Lee temperatura y humedad actuales. Usa el caché si sigue vigente,
/// salvo que se pida `force`.
pub fn fetch_weather(lat: f64, lon: f64, unit: &str, force: bool) -> Result<WeatherNow, String> {
    if !force {
        if let Some((when, data)) = *CACHE.lock().unwrap() {
            if when.elapsed() < CACHE_TTL { return Ok(data); }
        }
    }

    let unit_str = if unit == "imperial" { "fahrenheit" } else { "celsius" };
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m&temperature_unit={}",
        lat, lon, unit_str
    );
    let json: serde_json::Value = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call().map_err(|e| e.to_string())?
        .into_json().map_err(|e| e.to_string())?;

    let current = &json["current"];
    let data = WeatherNow {
        temp: current["temperature_2m"].as_f64().ok_or("Respuesta de clima inválida")?,
        hum:  current["relative_humidity_2m"].as_f64().ok_or("Respuesta de clima inválida")?,
    };
    *CACHE.lock().unwrap() = Some((Instant::now(), data));
    Ok(data)
}

/// Lee el clima de la configuración: si faltan coordenadas pero hay ciudad,
/// la geocodifica una vez y persiste las coordenadas (raíz del error
/// "Ciudad no configurada" cuando el usuario escribió la ciudad a mano).
pub fn weather_now(cfg_mutex: &Mutex<AppConfig>, force: bool) -> Result<WeatherNow, String> {
    let (mut lat, mut lon, unit, city, enabled) = {
        let cfg = cfg_mutex.lock().unwrap();
        let l = &cfg.locutions;
        (l.weather_lat, l.weather_lon, l.weather_unit.clone(), l.weather_city.clone(),
         cfg.weather_module_enabled && l.weather_enabled)
    };
    if !enabled { return Err("weather_disabled".to_string()); }
    if lat == 0.0 && lon == 0.0 {
        if city.is_empty() { return Err("no_city".to_string()); }
        let found = search_city(&city)?;
        let first = found.first().ok_or("city_not_found".to_string())?;
        lat = first.lat;
        lon = first.lon;
        let mut cfg = cfg_mutex.lock().unwrap();
        cfg.locutions.weather_lat = lat;
        cfg.locutions.weather_lon = lon;
        config::save_config(&cfg)?;
    }
    fetch_weather(lat, lon, &unit, force)
}

/// Hilo de refresco automático: cada 15 min consulta el clima (si el bloque
/// está activo) y emite "weather-updated" con los valores nuevos.
pub fn start_auto_refresh(app: tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    std::thread::spawn(move || loop {
        {
            let state = app.state::<crate::AppState>();
            if let Ok(now) = weather_now(&state.config, true) {
                let _ = app.emit("weather-updated", now);
            }
        }
        std::thread::sleep(AUTO_REFRESH);
    });
}

/// Codificación mínima de URL para el nombre de ciudad.
fn urlencode(s: &str) -> String {
    s.chars().map(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c.to_string(),
        ' ' => "+".to_string(),
        _   => c.to_string().bytes().map(|b| format!("%{:02X}", b)).collect(),
    }).collect()
}
