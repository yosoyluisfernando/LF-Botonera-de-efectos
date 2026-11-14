//! Pruebas del buscador global de efectos.
use super::search;
use crate::domain::button::defaults::new_button;
use crate::model::{PaletaData, ProfileData};

fn paleta(id: &str, nombre: &str, buttons: &[(u32, &str, &str)]) -> PaletaData {
    PaletaData {
        id: id.into(),
        nombre: nombre.into(),
        rows: 5,
        cols: 5,
        audio_out: String::new(),
        shortcut: String::new(),
        midi: Default::default(),
        tab_bg: String::new(),
        tab_text: String::new(),
        font_size: 0,
        botones: buttons
            .iter()
            .map(|(index, label, path)| {
                let mut b = new_button(id, *index, label, "#000000", "#FFFFFF");
                b.path = path.to_string();
                b
            })
            .collect(),
    }
}

fn profile() -> ProfileData {
    ProfileData {
        id: "p".into(),
        name: "Radio".into(),
        bg: String::new(),
        text: String::new(),
        audio: Default::default(),
        active_paleta_id: "a".into(),
        paletas: vec![
            paleta("a", "Ambientales", &[(1, "Lluvia", "C:/fx/lluvia.mp3"), (7, "Aplausos cortos", "C:/fx/ap1.wav")]),
            paleta("b", "Cortinas", &[(5, "Cortina noticias", "C:/fx/news.m4a"), (6, "Gran ovación", "C:/fx/APLAUSOS_publico.mp3")]),
        ],
        fixed_buttons: Vec::new(),
    }
}

#[test]
fn busca_en_todas_las_pestanas_por_nombre_y_archivo() {
    let hits = search(&profile(), "aplausos");
    let ids: Vec<_> = hits.iter().map(|h| h.button_id.as_str()).collect();
    assert_eq!(ids, ["a_btn_7", "b_btn_6"], "el nombre de archivo tambien cuenta");
    assert_eq!(hits[1].paleta_nombre, "Cortinas");
}

#[test]
fn ignora_tildes_y_mayusculas() {
    assert_eq!(search(&profile(), "OVACION").len(), 1);
    assert_eq!(search(&profile(), "lluvía").len(), 1);
}

#[test]
fn localiza_por_pestana_y_posicion() {
    let hits = search(&profile(), "cortinas 5");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].button_id, "b_btn_5");
    assert_eq!((hits[0].row, hits[0].col), (1, 5));
    assert_eq!(hits[0].tab_position, 2);
    let by_pos = search(&profile(), "7");
    assert_eq!(by_pos.len(), 1);
    assert_eq!((by_pos[0].row, by_pos[0].col), (2, 2));
}

#[test]
fn todas_las_palabras_deben_coincidir() {
    assert!(search(&profile(), "lluvia noticias").is_empty());
    assert!(search(&profile(), "   ").is_empty());
}

#[test]
fn primero_lo_que_empieza_por_la_consulta() {
    let hits = search(&profile(), "cort");
    assert_eq!(hits[0].button_id, "b_btn_5", "«Cortina…» empieza por la consulta");
}
