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

/// Cuántas candidatas se piden al resolver. Diez van sobradas: en los nombres
/// más repetidos (Barcelona, Valencia, Santiago, San José) la del país buscado
/// aparece siempre entre las dos primeras.
const RESOLVE_COUNT: u8 = 10;

/// Resuelve una ciudad a coordenadas.
///
/// La API busca por NOMBRE, no por etiqueta: mandarle "Barcelona, Estado
/// Anzoátegui, VE" entera devuelve cero resultados. Por eso se busca solo por el
/// nombre... pero el resto de la etiqueta **no se tira**, que era el bug: dice
/// CUÁL de las Barcelonas es, y sin ella siempre ganaba la más poblada.
pub fn resolve_coords(city: &str) -> Result<CityResult, String> {
    let name = city.split(',').next().unwrap_or(city).trim();
    if name.is_empty() {
        return Err("no_city".to_string());
    }
    let found = geocode(name, RESOLVE_COUNT)?;
    pick(&found, city)
        .cloned()
        .ok_or_else(|| "city_not_found".to_string())
}

/// Elige, de las candidatas, la que pide la etiqueta guardada. Aquí vive la
/// decisión y por eso está separada de la red: se puede probar sin ella.
///
/// Se afina de más a menos, y el último escalón es el comportamiento de siempre.
fn pick<'a>(found: &'a [CityResult], city: &str) -> Option<&'a CityResult> {
    // 1. La etiqueta entera. Sale del propio autocompletado, así que compararla
    //    con la que genera `geocode` es exacto: misma fuente, mismo formato.
    if let Some(exact) = found.iter().find(|c| c.label == city) {
        return Some(exact);
    }
    // 2. Solo el país. Cubre lo escrito a mano ("Valencia, VE") y las etiquetas
    //    viejas cuya región la API haya renombrado desde entonces.
    let country = country_of(city);
    if country != city {
        if let Some(same) = found.iter().find(|c| country_of(&c.label) == country) {
            return Some(same);
        }
    }
    // 3. Sin país que valga es ambigua de verdad: "Barcelona" a secas, o una
    //    configuración anterior a las etiquetas. La primera es lo único
    //    razonable, y es lo que se ha hecho siempre.
    found.first()
}

/// El país de una etiqueta: su último segmento. Sin coma no hay país, y
/// devuelve la cadena entera — así el llamante puede saber que no lo había.
fn country_of(label: &str) -> &str {
    label.rsplit(',').next().unwrap_or(label).trim()
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

#[cfg(test)]
#[path = "geocode_tests.rs"]
mod geocode_tests;
