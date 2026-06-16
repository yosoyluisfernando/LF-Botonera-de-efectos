/// Modulo: button_defaults.rs
/// Proposito: construir botones nuevos con valores consistentes.
use crate::types::ButtonData;

pub fn new_button(paleta_id: &str, index: u32, label: &str, bg: &str, text: &str) -> ButtonData {
    ButtonData {
        id: format!("{}_btn_{}", paleta_id, index),
        index,
        label: label.to_string(),
        type_field: "audio".to_string(),
        path: String::new(),
        folder: String::new(),
        name: label.to_string(),
        color_bg: bg.to_string(),
        color_text: text.to_string(),
        vol: 1.0,
        duration: -1.0,
        duration_str: String::new(),
        loop_mode: false,
        stop_other: false,
        overlap: false,
        restart: false,
        shortcut: String::new(),
    }
}
