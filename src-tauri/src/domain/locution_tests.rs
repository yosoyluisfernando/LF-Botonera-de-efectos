//! Pruebas de qué archivo dice qué. Van sin disco y sin reloj: lo que se prueba
//! es la DECISIÓN, y por eso `time_sequence` y `climate` reciben los nombres ya
//! leídos.
//!
//! Los nombres de los casos son los que de verdad reparten ZaraRadio, Salamandra
//! y RadioBOSS (comprobado contra su documentación el 2026-07-17).
use super::{climate, pick, time_sequence, Locution};

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// Lo que suena, en orden, a esa hora.
fn suenan(files: &[String], hh: u32, mm: u32) -> Vec<&str> {
    time_sequence(files, hh, mm)
        .iter()
        .map(|&i| files[i].as_str())
        .collect()
}

/// Lo que suena para un valor de clima, o "" si no hay archivo.
fn suena_clima<'a>(files: &'a [String], kind: &str, value: f64) -> &'a str {
    climate(files, kind, value).map_or("", |i| files[i].as_str())
}

/// Un pack de ZaraRadio completo: hora, "en punto" y minutos.
fn zara_hora() -> Vec<String> {
    names(&["HRS13.mp3", "HRS14.mp3", "HRS14_O.mp3", "MIN00.mp3", "MIN25.mp3"])
}

// ─── Hora ─────────────────────────────────────────────────────────────────────

#[test]
fn en_punto_gana_el_archivo_propio() {
    assert_eq!(suenan(&zara_hora(), 14, 0), ["HRS14_O.mp3"]);
}

/// El error más repetido del gremio: el cero en vez de la letra O. Suena igual y
/// no se distingue a simple vista, así que vale.
#[test]
fn en_punto_tambien_con_un_cero_en_vez_de_la_letra_o() {
    let files = names(&["HRS14.mp3", "HRS14_0.mp3", "MIN00.mp3"]);
    assert_eq!(suenan(&files, 14, 0), ["HRS14_0.mp3"]);
}

/// Un pack sin "en punto" —el de Salamandra, sin ir más lejos— decía la hora y
/// se callaba. Ahora cae a la hora y el minuto.
#[test]
fn sin_archivo_en_punto_se_dice_la_hora_y_el_minuto() {
    let files = names(&["HRS14.mp3", "MIN00.mp3"]);
    assert_eq!(suenan(&files, 14, 0), ["HRS14.mp3", "MIN00.mp3"]);
}

/// Y si tampoco hay MIN00, al menos se dice la hora. Callar es lo único peor.
#[test]
fn sin_en_punto_ni_min00_queda_la_hora_sola() {
    assert_eq!(suenan(&names(&["HRS14.mp3"]), 14, 0), ["HRS14.mp3"]);
}

#[test]
fn a_las_y_algo_suenan_la_hora_y_el_minuto() {
    assert_eq!(suenan(&zara_hora(), 14, 25), ["HRS14.mp3", "MIN25.mp3"]);
}

/// El "y…" no puede quedarse con el archivo del "en punto": diría "son las dos
/// en punto" y detrás "veinticinco".
#[test]
fn el_y_algo_no_se_lleva_el_archivo_del_en_punto() {
    let files = names(&["HRS14_O.mp3", "MIN25.mp3"]);
    assert_eq!(suenan(&files, 14, 25), ["MIN25.mp3"]);
}

#[test]
fn sin_ningun_archivo_de_esa_hora_no_suena_nada() {
    assert!(suenan(&names(&["HRS13.mp3"]), 14, 0).is_empty());
}

// ─── Rótulos y desempate ──────────────────────────────────────────────────────

/// Mucho pack viene rotulado. Mientras el número acabe donde acaba el prefijo,
/// vale.
#[test]
fn un_nombre_rotulado_sirve_igual() {
    let files = names(&["HRS14 - las dos.mp3"]);
    assert_eq!(suenan(&files, 14, 30), ["HRS14 - las dos.mp3"]);
}

/// Con los dos delante gana el exacto. Antes ganaba el que devolviera `read_dir`
/// primero, que no promete nada: la misma carpeta podía sonar distinta en dos
/// equipos.
#[test]
fn el_nombre_exacto_le_gana_al_rotulado() {
    let files = names(&["HRS14 - las dos.mp3", "HRS14.mp3"]);
    assert_eq!(suenan(&files, 14, 30), ["HRS14.mp3"]);
}

/// Y entre dos rotulados se elige siempre el mismo, no el que toque.
#[test]
fn entre_dos_rotulados_el_desempate_es_estable() {
    let unos = names(&["HRS14 - zeta.mp3", "HRS14 - alfa.mp3"]);
    let otros = names(&["HRS14 - alfa.mp3", "HRS14 - zeta.mp3"]);
    assert_eq!(suenan(&unos, 14, 30), ["HRS14 - alfa.mp3"]);
    assert_eq!(suenan(&otros, 14, 30), ["HRS14 - alfa.mp3"]);
}

#[test]
fn las_mayusculas_dan_igual() {
    assert_eq!(suenan(&names(&["hrs14_o.MP3"]), 14, 0), ["hrs14_o.MP3"]);
}

// ─── Temperatura ──────────────────────────────────────────────────────────────

#[test]
fn la_temperatura_de_zararadio_con_sus_tres_digitos() {
    let files = names(&["TMP024.mp3", "TMP025.mp3"]);
    assert_eq!(suena_clima(&files, "temperature", 25.0), "TMP025.mp3");
}

/// RadioBOSS no rellena con ceros: su manual da TMP29.mp3 tal cual.
#[test]
fn la_temperatura_de_radioboss_sin_ceros_delante() {
    let files = names(&["TMP29.mp3"]);
    assert_eq!(suena_clima(&files, "temperature", 29.0), "TMP29.mp3");
}

/// La trampa que obliga a mirar lo que sigue al número: a 0 grados, el alias
/// corto "TMP0" empieza igual que "TMP025" y la radio habría dicho veinticinco.
#[test]
fn a_cero_grados_no_se_cuela_la_de_veinticinco() {
    assert_eq!(suena_clima(&names(&["TMP025.mp3"]), "temperature", 0.0), "");
}

#[test]
fn bajo_cero_con_la_ene_de_zararadio() {
    let files = names(&["TMPN003.mp3"]);
    assert_eq!(suena_clima(&files, "temperature", -3.0), "TMPN003.mp3");
}

/// RadioBOSS escribe el signo: su manual da TMP-10.mp3.
#[test]
fn bajo_cero_con_el_signo_de_radioboss() {
    let files = names(&["TMP-10.mp3"]);
    assert_eq!(suena_clima(&files, "temperature", -10.0), "TMP-10.mp3");
}

/// Y -3 no puede llevarse el de -30, que empieza igual.
#[test]
fn bajo_cero_no_confunde_tres_con_treinta() {
    assert_eq!(suena_clima(&names(&["TMP-30.mp3"]), "temperature", -3.0), "");
}

/// El nombre es entero: los decimales del servicio de clima se redondean.
#[test]
fn los_decimales_se_redondean() {
    let files = names(&["TMP025.mp3"]);
    assert_eq!(suena_clima(&files, "temperature", 24.6), "TMP025.mp3");
}

// ─── Humedad ──────────────────────────────────────────────────────────────────

#[test]
fn la_humedad_con_y_sin_ceros_delante() {
    let zara = names(&["HUM082.mp3"]);
    let boss = names(&["HUM82.mp3"]);
    assert_eq!(suena_clima(&zara, "humidity", 82.0), "HUM082.mp3");
    assert_eq!(suena_clima(&boss, "humidity", 82.0), "HUM82.mp3");
}

/// El 100 % es el único donde los dos nombres coinciden, y no debe duplicarse.
#[test]
fn la_humedad_del_cien_por_cien_tiene_un_solo_nombre() {
    assert_eq!(Locution::Humidity(100).aliases(), ["HUM100"]);
}

/// Un servicio que devuelva 105 no rompe: la humedad no pasa del 100.
#[test]
fn la_humedad_se_queda_dentro_de_su_rango() {
    let files = names(&["HUM100.mp3"]);
    assert_eq!(suena_clima(&files, "humidity", 105.0), "HUM100.mp3");
}

// ─── Alias ────────────────────────────────────────────────────────────────────

/// Fija el orden en que se prueban: primero lo canónico de ZaraRadio.
#[test]
fn los_alias_van_del_canonico_al_tolerado() {
    assert_eq!(Locution::Temperature(5).aliases(), ["TMP005", "TMP5"]);
    assert_eq!(
        Locution::Temperature(-5).aliases(),
        ["TMPN005", "TMPN5", "TMP-005", "TMP-5"]
    );
    assert_eq!(Locution::HourSharp(9).aliases(), ["HRS09_O", "HRS09_0"]);
}

#[test]
fn sin_carpeta_no_hay_archivo() {
    assert!(pick(&[], &Locution::Hour(14)).is_none());
}
