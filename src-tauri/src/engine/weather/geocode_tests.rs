//! Pruebas de a cuál de las ciudades homónimas se le hace caso. Van sin red: lo
//! que se prueba es la DECISIÓN, no la llamada — por eso `pick` está separada.
//!
//! Las candidatas son las que devuelve Open-Meteo de verdad, en su orden real
//! (comprobado contra la API el 2026-07-16). El primero de la lista es siempre
//! el más poblado, y ahí estaba el bug: se cogía ese y nada más.
use super::{pick, CityResult};

fn city(label: &str, lat: f64, lon: f64) -> CityResult {
    CityResult {
        label: label.to_string(),
        lat,
        lon,
    }
}

/// Las cinco Barcelonas, tal cual las ordena Open-Meteo.
fn barcelonas() -> Vec<CityResult> {
    vec![
        city("Barcelona, Comunidad Autónoma de Cataluña, ES", 41.38879, 2.15899),
        city("Barcelona, Estado Anzoátegui, VE", 10.1384, -64.68769),
        city("Barcelona, Bicolandia, PH", 12.8683, 124.1419),
        city("Barcelona, Estado de Río Grande del Norte, BR", -5.95056, -35.92639),
    ]
}

/// El caso que reportó el autor: elegir Venezuela en el desplegable daba España.
#[test]
fn la_ciudad_elegida_gana_a_la_mas_poblada() {
    let candidatas = barcelonas();
    let elegida = pick(&candidatas, "Barcelona, Estado Anzoátegui, VE").unwrap();
    assert_eq!(elegida.label, "Barcelona, Estado Anzoátegui, VE");
    assert!((elegida.lat - 10.1384).abs() < 0.001, "lat: {}", elegida.lat);
}

/// Y la de España sigue saliendo cuando es la que se pidió: el arreglo no puede
/// consistir en invertir el problema.
#[test]
fn la_mas_poblada_gana_cuando_es_la_pedida() {
    let candidatas = barcelonas();
    let elegida = pick(&candidatas, "Barcelona, Comunidad Autónoma de Cataluña, ES").unwrap();
    assert_eq!(elegida.label, "Barcelona, Comunidad Autónoma de Cataluña, ES");
}

/// Escrito a mano, sin la región exacta: con el país basta para desempatar.
#[test]
fn con_el_pais_basta_aunque_falte_la_region() {
    let candidatas = barcelonas();
    let elegida = pick(&candidatas, "Barcelona, VE").unwrap();
    assert_eq!(elegida.label, "Barcelona, Estado Anzoátegui, VE");
}

/// Una región que la API haya renombrado desde que se guardó la etiqueta: cae al
/// país, que es lo que de verdad desambigua.
#[test]
fn una_region_que_ya_no_cuadra_cae_al_pais() {
    let candidatas = barcelonas();
    let elegida = pick(&candidatas, "Barcelona, Anzoátegui, VE").unwrap();
    assert_eq!(elegida.label, "Barcelona, Estado Anzoátegui, VE");
}

/// Sin país no hay nada que decidir: es ambigua de verdad. Se coge la primera,
/// que es lo que se hacía siempre — configuraciones anteriores a las etiquetas.
#[test]
fn sin_pais_es_ambigua_y_se_coge_la_primera() {
    let candidatas = barcelonas();
    let elegida = pick(&candidatas, "Barcelona").unwrap();
    assert_eq!(elegida.label, "Barcelona, Comunidad Autónoma de Cataluña, ES");
}

/// El país pedido no está entre las candidatas (una etiqueta inventada, un país
/// mal escrito): no se inventa nada, se cae a la primera.
#[test]
fn un_pais_que_no_esta_cae_a_la_primera() {
    let candidatas = barcelonas();
    let elegida = pick(&candidatas, "Barcelona, XX").unwrap();
    assert_eq!(elegida.label, "Barcelona, Comunidad Autónoma de Cataluña, ES");
}

/// Sin candidatas no hay ciudad. El llamante lo traduce a `city_not_found`.
#[test]
fn sin_candidatas_no_hay_ciudad() {
    assert!(pick(&[], "Barcelona, VE").is_none());
}

/// El Callao: aquí el homónimo está en OTRO país, y el nombre lleva artículo.
/// Buscar "Callao" no lo encuentra —la API busca por prefijo—, pero eso es cosa
/// de qué se escribe; una vez en la lista, se elige bien.
#[test]
fn el_callao_de_bolivar_gana_al_de_peru() {
    let callaos = vec![
        city("El Callao, Provincia Constitucional del Callao, PE", -12.05162, -77.13452),
        city("El Callao, Estado Bolívar, VE", 7.34706, -61.82684),
        city("El Callao, Estado Lara, VE", 10.29121, -69.23874),
    ];
    let elegida = pick(&callaos, "El Callao, Estado Bolívar, VE").unwrap();
    assert_eq!(elegida.label, "El Callao, Estado Bolívar, VE");
    // Y por país a secas sale el primer venezolano, no el peruano.
    assert_eq!(
        pick(&callaos, "El Callao, VE").unwrap().label,
        "El Callao, Estado Bolívar, VE"
    );
}
