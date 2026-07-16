//! Pruebas de las migraciones que se aplican al cargar la configuracion.
use super::*;
use crate::model::ButtonData;

fn track(kind: &str, folder: &str) -> ButtonData {
    let mut b = crate::domain::button::defaults::new_button("player", 1, "x", "#000", "#fff");
    b.type_field = kind.into();
    b.folder = folder.into();
    b
}

/// Las listas del LFA importadas antes del arreglo guardaron el marcador como
/// carpeta, y la locucion no sonaba. Al cargar se limpia solo: el operador no
/// tiene que reimportar nada.
#[test]
fn locution_markers_saved_as_folders_are_cleared() {
    let mut cfg = AppConfig::default();
    cfg.player.tracks = vec![
        track("time", "time_locution"),
        track("temperature", "temperature_locution"),
        track("humidity", "humidity_locution"),
    ];

    clear_locution_markers(&mut cfg);

    for t in &cfg.player.tracks {
        assert_eq!(t.folder, "", "{} deberia quedar sin carpeta", t.type_field);
    }
}

/// Una carpeta de verdad NO se toca: la botonera admite carpeta por fila.
#[test]
fn a_real_folder_is_never_cleared() {
    let mut cfg = AppConfig::default();
    cfg.player.tracks = vec![track("time", r"C:\Locuciones\Hora")];

    clear_locution_markers(&mut cfg);

    assert_eq!(cfg.player.tracks[0].folder, r"C:\Locuciones\Hora");
}

/// Y una carpeta aleatoria que se llamara asi por casualidad tampoco: la
/// limpieza solo aplica a los tipos de locucion.
#[test]
fn only_locution_types_are_touched() {
    let mut cfg = AppConfig::default();
    cfg.player.tracks = vec![track("random_folder", "time_locution")];

    clear_locution_markers(&mut cfg);

    assert_eq!(cfg.player.tracks[0].folder, "time_locution");
}
