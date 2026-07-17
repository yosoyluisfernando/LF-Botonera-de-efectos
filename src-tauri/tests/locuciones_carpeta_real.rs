//! Prueba el resolver de locuciones contra CARPETAS DE VERDAD, con los packs de
//! ZaraRadio, Salamandra y RadioBOSS montados tal como los reparten ellos.
//!
//! Las pruebas de `domain::locution` deciden sobre una lista de nombres en
//! memoria. Estas escriben los archivos en el disco y comprueban la tubería
//! entera: leer la carpeta, elegir, y devolver una ruta que existe de verdad. Es
//! lo único que puede cazar un fallo de `read_dir`, que es donde estaba el bug
//! del orden indefinido.
//!
//! NO llevan `#[ignore]`: no necesitan tarjeta de sonido ni red, solo un
//! directorio temporal. Entran en la suite normal.
//!
//! ```text
//! cargo test --test locuciones_carpeta_real
//! ```
mod locuciones;

use locuciones::*;
use tauri_app_lib::engine::weather::resolver::{resolve_climate_file, resolve_time_files};

// ─── Hora ─────────────────────────────────────────────────────────────────────

#[test]
fn la_hora_de_zararadio_sale_de_una_carpeta_de_verdad() {
    let carpeta = Carpeta::con(&pack_zara_hora());
    let ((hh, mm), suena) = resuelve_con_el_reloj_quieto(&carpeta);
    assert_eq!(suena, hora_esperada(hh, mm));
}

/// El pack de Salamandra no trae "en punto": a y algo suena la hora y el minuto,
/// y al minuto 00 también, en vez de callarse como antes. El jingle nunca sale:
/// no empieza por ningún código.
#[test]
fn la_hora_de_salamandra_suena_y_su_jingle_no_estorba() {
    let carpeta = Carpeta::con(&pack_salamandra_hora());
    let ((hh, mm), suena) = resuelve_con_el_reloj_quieto(&carpeta);
    assert_eq!(suena, [format!("HRS{hh:02}.mp3"), format!("MIN{mm:02}.mp3")]);
}

#[test]
fn un_pack_rotulado_sirve_sin_tocarle_el_nombre() {
    let carpeta = Carpeta::con(&pack_rotulado());
    let ((hh, mm), suena) = resuelve_con_el_reloj_quieto(&carpeta);
    assert_eq!(
        suena,
        [
            format!("HRS{hh:02} - son las {hh}.mp3"),
            format!("MIN{mm:02} - y {mm}.mp3")
        ]
    );
}

/// El bug del orden: con el exacto y el rotulado en la MISMA carpeta, `read_dir`
/// devolvía uno u otro según le parecía, y la misma carpeta sonaba distinto en
/// dos equipos. Aquí están los dos, en disco, y tiene que ganar el exacto.
#[test]
fn en_disco_el_nombre_exacto_le_gana_al_rotulado() {
    let mut archivos = pack_zara_hora();
    archivos.extend(pack_rotulado());
    let carpeta = Carpeta::con(&archivos);
    let ((hh, mm), suena) = resuelve_con_el_reloj_quieto(&carpeta);
    assert_eq!(suena, hora_esperada(hh, mm));
}

/// Con un pack completo: en punto suena su archivo, y si no, la hora y el minuto.
fn hora_esperada(hh: u32, mm: u32) -> Vec<String> {
    if mm == 0 {
        vec![format!("HRS{hh:02}_O.mp3")]
    } else {
        vec![format!("HRS{hh:02}.mp3"), format!("MIN{mm:02}.mp3")]
    }
}

// ─── Clima ────────────────────────────────────────────────────────────────────

#[test]
fn el_clima_de_zararadio_sale_de_una_carpeta_de_verdad() {
    let carpeta = Carpeta::con(&pack_zara_clima());
    assert_eq!(carpeta.clima("temperature", 25.0), "TMP025.mp3");
    assert_eq!(carpeta.clima("temperature", -3.0), "TMPN003.mp3");
    assert_eq!(carpeta.clima("humidity", 82.0), "HUM082.mp3");
    assert_eq!(carpeta.clima("temperature", 0.0), "TMP000.mp3");
}

/// Los tres ejemplos son los del manual de RadioBOSS, sin tocarles una letra.
#[test]
fn el_clima_de_radioboss_suena_sin_renombrar_nada() {
    let carpeta = Carpeta::con(&pack_radioboss_clima());
    assert_eq!(carpeta.clima("temperature", 29.0), "TMP29.mp3");
    assert_eq!(carpeta.clima("temperature", -10.0), "TMP-10.mp3");
    assert_eq!(carpeta.clima("humidity", 3.0), "HUM3.mp3");
    assert_eq!(carpeta.clima("temperature", 0.0), "TMP0.mp3");
}

/// La trampa, ahora con los archivos en el disco: en un pack de ZaraRadio sin la
/// de cero grados, "TMP025.mp3" empieza por "TMP0". No puede colarse.
#[test]
fn a_cero_grados_no_se_cuela_la_de_veinticinco() {
    let carpeta = Carpeta::con(&["TMP025.mp3".to_string()]);
    assert_eq!(
        resolve_climate_file(carpeta.path(), "temperature", 0.0),
        Err("no_climate_locution".to_string())
    );
}

// ─── Lo que falta se dice con una clave, no con una frase en español ──────────

#[test]
fn una_carpeta_que_no_existe_lo_dice_con_su_clave() {
    let fantasma = std::env::temp_dir().join("lf_botonera_carpeta_que_no_existe");
    let ruta = fantasma.to_string_lossy().to_string();
    assert_eq!(
        resolve_time_files(&ruta),
        Err("locution_folder_missing".to_string())
    );
    assert_eq!(
        resolve_climate_file(&ruta, "temperature", 20.0),
        Err("locution_folder_missing".to_string())
    );
}

#[test]
fn una_carpeta_sin_los_archivos_lo_dice_con_su_clave() {
    let carpeta = Carpeta::con(&[]);
    assert_eq!(
        resolve_time_files(carpeta.path()),
        Err("no_time_locution".to_string())
    );
}
