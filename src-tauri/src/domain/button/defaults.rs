/// Modulo: button_defaults.rs
/// Proposito: construir botones nuevos con valores consistentes.
use crate::model::ButtonData;

pub fn new_button(paleta_id: &str, index: u32, label: &str, bg: &str, text: &str) -> ButtonData {
    ButtonData {
        obs: None,
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
        midi: Default::default(),
        visual: Default::default(),
        style: Default::default(),
    }
}

/// Solo una celda nueva recibe colores aleatorios. Reemplazar su contenido
/// conserva sus colores y estilo, independientemente del tipo de sonido.
pub fn new_button_preserving_colors(
    prefix: &str,
    index: u32,
    label: &str,
    theme: &str,
    previous: Option<&ButtonData>,
) -> ButtonData {
    let (bg, text) = match previous {
        Some(old) => (old.color_bg.clone(), old.color_text.clone()),
        None => {
            let bg = crate::domain::colors::random_color();
            let text = crate::domain::colors::text_for_theme(&bg, theme, "button");
            (bg, text)
        }
    };
    let mut button = new_button(prefix, index, label, &bg, &text);
    if let Some(old) = previous {
        button.style = old.style.clone();
    }
    button
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacement_preserves_custom_and_predefined_colors_for_all_content_types() {
        for original_type in ["audio", "random_folder", "time", "temperature", "humidity"] {
            let mut old = new_button("p", 1, "Antes", "#598042", "#FFD700");
            old.type_field = original_type.into();
            old.style = crate::model::ButtonStyle {
                gradient: "diagonal_inv".into(),
                bg2: "#021927".into(),
                text2: "#FFFFFF".into(),
                exact: true,
            };
            for theme in ["dark", "light"] {
                let new = new_button_preserving_colors("p", 1, "Otro sonido", theme, Some(&old));
                assert_eq!(new.color_bg, old.color_bg);
                assert_eq!(new.color_text, old.color_text);
                assert_eq!(new.style, old.style);
                assert_eq!(new.label, "Otro sonido");
            }
            old.style = Default::default();
            assert!(
                new_button_preserving_colors("p", 1, "Nuevo", "light", Some(&old))
                    .style
                    .is_default()
            );
        }
    }
    #[test]
    fn only_new_cells_choose_a_random_predefined_color() {
        let new = new_button_preserving_colors("p", 1, "Audio", "dark", None);
        assert!(crate::domain::colors::color_palette()
            .iter()
            .any(|c| c.base == new.color_bg));
        assert!(new.style.is_default());
    }
}
