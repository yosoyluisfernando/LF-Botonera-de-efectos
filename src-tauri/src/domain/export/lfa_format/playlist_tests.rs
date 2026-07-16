//! Pruebas del adaptador `.LFPlay`. Lo que se comprueba aqui es la
//! COMPATIBILIDAD BIDIRECCIONAL con LF Automatizador: sus variantes de
//! formato deben entrar, y lo que exportamos debe poder abrirlo el.
use super::*;

/// Fila tal cual la escribe el Automatizador: trae campos suyos que no nos
/// incumben (eventId, temp, recursive...) y la duracion como entero.
const LFA_ROW: &str = r#"{
    "ruta": "D:\\Music\\DEMBOW\\cancion.mp3",
    "titulo": "Alayon - La quiero con bendicion",
    "duracion": 172,
    "type": "normal",
    "target": null,
    "eventId": "",
    "eventName": "",
    "temp": false,
    "fileType": null,
    "recursive": false,
    "resolvedRandomPath": ""
}"#;

#[test]
fn lfa_row_keeps_duration_and_ignores_foreign_fields() {
    let row: LfaPlaylistRow = serde_json::from_str(LFA_ROW).expect("debe deserializar");
    assert_eq!(row.duracion, 172.0);
    let btn = from_lfa_row(&row, 1, "#000", "#fff").expect("normal -> audio");
    assert_eq!(btn.duration, 172.0);
    assert_eq!(btn.type_field, "audio");
}

/// El Automatizador acepta `duracion` o `duration` (ver su
/// `auxiliary_playlists.js`), asi que hay listas suyas con el nombre ingles.
#[test]
fn lfa_row_accepts_english_duration_alias() {
    let row: LfaPlaylistRow =
        serde_json::from_str(r#"{"ruta":"a.mp3","duration":90,"type":"normal"}"#)
            .expect("debe deserializar");
    assert_eq!(row.duracion, 90.0);
}

/// El Automatizador guarda la duracion unas veces como numero y otras como
/// CADENA (por eso el suyo la lee con `parseInt`). Hay que aceptar ambas: si
/// no, una sola fila con `"31"` tumbaba el archivo entero al deserializar.
#[test]
fn lfa_row_accepts_duration_as_string_or_number() {
    let as_string: LfaPlaylistRow =
        serde_json::from_str(r#"{"ruta":"a.mp3","duracion":"31","type":"normal"}"#)
            .expect("cadena debe deserializar");
    assert_eq!(as_string.duracion, 31.0);

    let as_number: LfaPlaylistRow =
        serde_json::from_str(r#"{"ruta":"a.mp3","duracion":172,"type":"normal"}"#)
            .expect("numero debe deserializar");
    assert_eq!(as_number.duracion, 172.0);
}

/// Caso real: el LFA escribe `ruta: "time_locution"`, un MARCADOR, no una
/// carpeta. Tomarlo por carpeta hacia buscar un directorio con ese nombre y
/// la locucion no sonaba nunca. Debe quedar sin carpeta para que la botonera
/// use la suya de Ajustes, que es como resuelve el LFA con las suyas.
#[test]
fn lfa_locution_markers_are_not_folders() {
    for (kind, marker) in [
        ("time", "time_locution"),
        ("temperature", "temperature_locution"),
        ("humidity", "humidity_locution"),
    ] {
        let json = format!(r#"{{"ruta":"{marker}","type":"{kind}","duracion":5}}"#);
        let row: LfaPlaylistRow = serde_json::from_str(&json).unwrap();
        let btn = from_lfa_row(&row, 1, "#000", "#fff").expect("debe importarse");
        assert_eq!(btn.type_field, kind);
        assert_eq!(btn.folder, "", "{marker} no es una carpeta");
    }
}

/// Una carpeta propia SI se conserva: la botonera admite carpeta por fila.
#[test]
fn a_real_locution_folder_is_kept() {
    let row: LfaPlaylistRow =
        serde_json::from_str(r#"{"ruta":"C:\\Locuciones\\Hora","type":"time"}"#).unwrap();
    let btn = from_lfa_row(&row, 1, "#000", "#fff").unwrap();
    assert_eq!(btn.folder, r"C:\Locuciones\Hora");
}

/// Al exportar, una locucion sin carpeta lleva el marcador: una `ruta` vacia
/// le dejaria al LFA una fila que no sabria resolver.
#[test]
fn exporting_a_locution_without_folder_writes_the_marker() {
    let mut btn = new_button("player", 1, "Hora", "#000", "#fff");
    btn.type_field = "time".into();
    assert_eq!(to_lfa_row(&btn).ruta, "time_locution");

    btn.type_field = "humidity".into();
    assert_eq!(to_lfa_row(&btn).ruta, "humidity_locution");
}

/// Ida y vuelta: lo que exportamos debe volver a entrar igual.
#[test]
fn locution_survives_a_round_trip_through_the_lfa_format() {
    let mut btn = new_button("player", 1, "Hora", "#000", "#fff");
    btn.type_field = "time".into();
    let row = to_lfa_row(&btn);
    let back = from_lfa_row(&row, 1, "#000", "#fff").expect("debe volver");
    assert_eq!(back.type_field, "time");
    assert_eq!(back.folder, "", "sigue sin carpeta: la pone Ajustes");
}

/// Una duracion ilegible no puede tumbar la lista: vale mas la cancion sin
/// duracion que perder el archivo entero.
#[test]
fn lfa_row_survives_unreadable_duration() {
    let row: LfaPlaylistRow =
        serde_json::from_str(r#"{"ruta":"a.mp3","duracion":"","type":"normal"}"#)
            .expect("no debe reventar");
    assert_eq!(row.duracion, 0.0);
}
