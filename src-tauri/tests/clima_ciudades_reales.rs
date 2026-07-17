//! Prueba el geocodificador REAL contra la API de Open-Meteo, con las ciudades
//! homónimas que el autor reportó (2026-07-16).
//!
//! Marcada `#[ignore]` porque necesita red: en un equipo sin conexión —o en CI—
//! no puede pasar, y una prueba que falla por el entorno no dice nada. Se pide a
//! mano:
//!
//! ```text
//! cargo test --test clima_ciudades_reales -- --ignored --nocapture
//! ```
//!
//! Lo que comprueba, y que ninguna prueba sin red puede: que las candidatas que
//! Open-Meteo devuelve HOY siguen permitiendo distinguir estas ciudades. Si la
//! API cambiara su orden o sus etiquetas, aquí se vería.
use tauri_app_lib::engine::weather::geocode::resolve_coords;

/// Cada caso: la etiqueta guardada y las coordenadas que DEBE dar.
/// Antes del arreglo, las tres venezolanas daban España o Perú.
const CASOS: &[(&str, f64, f64)] = &[
    ("Barcelona, Estado Anzoátegui, VE", 10.1384, -64.68769),
    ("Valencia, Estado Carabobo, VE", 10.16153, -68.00044),
    ("Barcelona, Comunidad Autónoma de Cataluña, ES", 41.38879, 2.15899),
    ("Valencia, Comunidad Valenciana, ES", 39.47391, -0.37966),
    ("El Callao, Estado Bolívar, VE", 7.34706, -61.82684),
    (
        "El Callao, Provincia Constitucional del Callao, PE",
        -12.05162,
        -77.13452,
    ),
];

#[test]
#[ignore]
fn cada_ciudad_homonima_resuelve_a_la_suya() {
    let mut fallos = Vec::new();
    for (etiqueta, lat, lon) in CASOS {
        match resolve_coords(etiqueta) {
            Ok(c) => {
                // Un cuarto de grado: la API puede afinar sus coordenadas sin
                // que deje de ser la misma ciudad. Basta y sobra para saber que
                // no nos coló otra a 8.000 km.
                let bien = (c.lat - lat).abs() < 0.25 && (c.lon - lon).abs() < 0.25;
                println!(
                    "  {} {}\n      → {}  ({}, {})",
                    if bien { "✓" } else { "✗" },
                    etiqueta,
                    c.label,
                    c.lat,
                    c.lon
                );
                if !bien {
                    fallos.push(format!("{etiqueta} → {} ({}, {})", c.label, c.lat, c.lon));
                }
            }
            Err(e) => {
                println!("  ✗ {etiqueta}\n      → error: {e}");
                fallos.push(format!("{etiqueta} → {e}"));
            }
        }
    }
    assert!(fallos.is_empty(), "no resolvieron a su ciudad: {fallos:#?}");
}

/// Sin país la etiqueta es ambigua de verdad, y se coge la más poblada. No es un
/// fallo: es lo único razonable, y es lo que se hacía siempre. Se fija aquí para
/// que quede claro que el arreglo NO adivina.
#[test]
#[ignore]
fn una_ciudad_sin_pais_sigue_dando_la_mas_poblada() {
    let c = resolve_coords("Barcelona").expect("Barcelona debería resolver");
    println!("  \"Barcelona\" (ambigua) → {}", c.label);
    assert!(
        c.label.ends_with(", ES"),
        "sin país se coge la primera de Open-Meteo, que es España: {}",
        c.label
    );
}
