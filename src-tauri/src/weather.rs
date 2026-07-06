/// Módulo: weather.rs
/// Propósito: Cliente de clima de Open-Meteo (coordenadas → temperatura /
/// humedad), caché en memoria y refresco automático cada 15 minutos (como el
/// LFA) que emite el evento "weather-updated" hacia la UI. La geocodificación
/// (nombre → coordenadas) vive en geocode.rs.
use crate::config;
use crate::geocode;
use crate::model::AppConfig;
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
    pub hum: f64,
}

/// Resultado de la prueba "Comprobar": clima + la ciudad realmente resuelta.
#[derive(Serialize, Clone)]
pub struct WeatherPreview {
    pub temp: f64,
    pub hum: f64,
    pub label: String,
    pub lat: f64,
    pub lon: f64,
}

static CACHE: Mutex<Option<(Instant, WeatherNow)>> = Mutex::new(None);

/// Consulta directa a la API de clima (sin tocar el caché).
fn fetch_current(lat: f64, lon: f64, unit: &str) -> Result<WeatherNow, String> {
    let unit_str = if unit == "imperial" {
        "fahrenheit"
    } else {
        "celsius"
    };
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m&temperature_unit={}",
        lat, lon, unit_str
    );
    let json: serde_json::Value = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|_| "offline".to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    let current = &json["current"];
    Ok(WeatherNow {
        temp: current["temperature_2m"]
            .as_f64()
            .ok_or("Respuesta de clima inválida")?,
        hum: current["relative_humidity_2m"]
            .as_f64()
            .ok_or("Respuesta de clima inválida")?,
    })
}

/// Lee temperatura y humedad actuales. Usa el caché si sigue vigente,
/// salvo que se pida `force`.
pub fn fetch_weather(lat: f64, lon: f64, unit: &str, force: bool) -> Result<WeatherNow, String> {
    if !force {
        if let Some((when, data)) = *CACHE.lock().unwrap() {
            if when.elapsed() < CACHE_TTL {
                return Ok(data);
            }
        }
    }
    let data = fetch_current(lat, lon, unit)?;
    *CACHE.lock().unwrap() = Some((Instant::now(), data));
    Ok(data)
}

/// Prueba directa "esta ciudad da clima ahora": geocodifica y consulta sin
/// tocar la configuración, sin validar carpetas y sin usar el caché. Respaldo
/// del botón "Comprobar" del panel.
pub fn preview_weather(city: &str, unit: &str) -> Result<WeatherPreview, String> {
    let found = geocode::resolve_coords(city)?;
    let now = fetch_current(found.lat, found.lon, unit)?;
    Ok(WeatherPreview {
        temp: now.temp,
        hum: now.hum,
        label: found.label,
        lat: found.lat,
        lon: found.lon,
    })
}

/// Lee el clima de la configuración: si faltan coordenadas pero hay ciudad,
/// la resuelve una vez y persiste las coordenadas (fuente única en Rust).
pub fn weather_now(cfg_mutex: &Mutex<AppConfig>, force: bool) -> Result<WeatherNow, String> {
    let (mut lat, mut lon, unit, city, enabled) = {
        let cfg = cfg_mutex.lock().unwrap();
        let l = &cfg.locutions;
        (
            l.weather_lat,
            l.weather_lon,
            l.weather_unit.clone(),
            l.weather_city.clone(),
            cfg.weather_module_enabled && l.weather_enabled,
        )
    };
    if !enabled {
        return Err("weather_disabled".to_string());
    }
    if lat == 0.0 && lon == 0.0 {
        if city.is_empty() {
            return Err("no_city".to_string());
        }
        let found = geocode::resolve_coords(&city)?;
        lat = found.lat;
        lon = found.lon;
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
