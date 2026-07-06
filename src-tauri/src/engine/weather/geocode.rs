/// Módulo: geocode.rs
/// Propósito: Único dueño de la geocodificación (nombre de ciudad → coordenadas)
/// usando la API de Open-Meteo, igual que el LF Automatizador. La UI nunca
/// resuelve ni cachea coordenadas: solo pide aquí y pinta lo que devuelve Rust.
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct CityResult {
    pub label: String,
    pub lat: f64,
    pub lon: f64,
}

/// Llama a la geocodificación de Open-Meteo y devuelve hasta `count` ciudades.
fn geocode(name: &str, count: u8) -> Result<Vec<CityResult>, String> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count={}&language=es&format=json",
        urlencode(name),
        count
    );
    let json: serde_json::Value = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|_| "offline".to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(results) = json["results"].as_array() {
        for r in results {
            let name = r["name"].as_str().unwrap_or_default();
            let admin = r["admin1"]
                .as_str()
                .map(|a| format!(", {}", a))
                .unwrap_or_default();
            let country = r["country_code"].as_str().unwrap_or_default();
            out.push(CityResult {
                label: format!("{}{}, {}", name, admin, country),
                lat: r["latitude"].as_f64().unwrap_or(0.0),
                lon: r["longitude"].as_f64().unwrap_or(0.0),
            });
        }
    }
    Ok(out)
}

/// Busca ciudades por nombre para el autocompletado (hasta 5 sugerencias).
pub fn search_city(query: &str) -> Result<Vec<CityResult>, String> {
    geocode(query, 5)
}

/// Resuelve una ciudad a coordenadas. Usa solo el primer segmento antes de la
/// coma ("Madrid" de "Madrid, Comunidad de Madrid, ES"): así la etiqueta
/// completa del autocompletado nunca devuelve cero resultados (raíz del antiguo
/// "no existe la ciudad que el propio servidor autocompletó").
pub fn resolve_coords(city: &str) -> Result<CityResult, String> {
    let name = city.split(',').next().unwrap_or(city).trim();
    if name.is_empty() {
        return Err("no_city".to_string());
    }
    geocode(name, 1)?
        .into_iter()
        .next()
        .ok_or_else(|| "city_not_found".to_string())
}

/// Codificación mínima de URL para el nombre de ciudad.
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c.to_string(),
            ' ' => "+".to_string(),
            _ => c
                .to_string()
                .bytes()
                .map(|b| format!("%{:02X}", b))
                .collect(),
        })
        .collect()
}
