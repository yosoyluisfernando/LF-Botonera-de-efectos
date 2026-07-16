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

/// Una duracion que ya se leyo bien NO se vuelve a sondear: cuesta ~40 ms por
/// archivo y con una lista larga seria un arranque lento sin motivo.
#[test]
fn a_good_duration_is_never_probed_again() {
    let mut cfg = AppConfig::default();
    let mut t = track("audio", "");
    t.path = r"C:\no\existe\nada.mp3".into(); // si lo sondeara, daria -1
    t.duration = 123.0;
    cfg.player.tracks = vec![t];

    recover_missing_durations(&mut cfg);

    assert_eq!(cfg.player.tracks[0].duration, 123.0, "no se toca lo que ya estaba bien");
}

/// Un archivo que ya no existe deja la duracion como estaba: no se puede
/// recuperar, pero tampoco se estropea nada.
#[test]
fn a_missing_file_leaves_it_alone() {
    let mut cfg = AppConfig::default();
    let mut t = track("audio", "");
    t.path = r"C:\no\existe\nada.mp3".into();
    t.duration = -1.0;
    cfg.player.tracks = vec![t];

    recover_missing_durations(&mut cfg);

    assert_eq!(cfg.player.tracks[0].duration, -1.0);
}

/// Los tipos especiales no tienen duracion propia: no hay nada que sondear.
#[test]
fn special_types_are_not_probed() {
    let mut cfg = AppConfig::default();
    let mut t = track("random_folder", "C:/musica");
    t.duration = -1.0;
    cfg.player.tracks = vec![t];

    recover_missing_durations(&mut cfg);

    assert_eq!(cfg.player.tracks[0].duration, -1.0, "una carpeta no tiene duracion");
}

/// El agujero que casi se escapa: la migracion cubria solo la cola del
/// reproductor. Los BOTONES arrastran el fallo desde mucho antes, y
/// `get_grid_state` solo reintenta cuando la duracion vale `0`, no `-1`: un
/// boton atascado en `-1` no se recuperaria nunca.
#[test]
fn it_also_recovers_buttons_not_just_the_player_queue() {
    let wav = write_test_wav();
    let path = wav.to_string_lossy().to_string();
    let mut cfg = AppConfig::default();
    let broken = |name: &str| {
        let mut b = crate::domain::button::defaults::new_button("p", 1, name, "#000", "#fff");
        b.path = path.clone();
        b.duration = -1.0; // como si la etiqueta rota hubiera tumbado la lectura
        b
    };
    cfg.profiles[0].paletas[0].botones.push(broken("rejilla"));
    cfg.profiles[0].fixed_buttons.push(broken("fijo del perfil"));
    cfg.fixed_panel.global_buttons.push(broken("fijo global"));
    cfg.player.tracks.push(broken("cola"));

    recover_missing_durations(&mut cfg);
    let _ = std::fs::remove_file(&wav);

    // Los CUATRO sitios donde puede haber duracion, no solo la cola.
    assert!(cfg.profiles[0].paletas[0].botones[0].duration > 0.0, "rejilla sin recuperar");
    assert!(cfg.profiles[0].fixed_buttons[0].duration > 0.0, "fijo del perfil sin recuperar");
    assert!(cfg.fixed_panel.global_buttons[0].duration > 0.0, "fijo global sin recuperar");
    assert!(cfg.player.tracks[0].duration > 0.0, "cola sin recuperar");
}

/// WAV de un segundo, generado al vuelo: la prueba no depende de ningun archivo
/// del equipo.
fn write_test_wav() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("lf_dur_{}.wav", std::process::id()));
    let samples = vec![0i16; 8000];
    let mut b = Vec::new();
    let data_len = (samples.len() * 2) as u32;
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&8000u32.to_le_bytes());
    b.extend_from_slice(&16000u32.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in &samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(&path, b).unwrap();
    path
}
