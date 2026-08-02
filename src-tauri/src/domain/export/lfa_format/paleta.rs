use super::types::{default_out, default_text, default_type, LfaButton, LfaPaleta};
use crate::domain::colors::random_color;
use crate::model::{ButtonData, PaletaData};

/// Exporta la cuadricula completa: una entrada por celda, vacias incluidas.
/// El LFA empareja botones por posicion del array, no por id.
pub fn to_lfa_paleta(p: &PaletaData) -> LfaPaleta {
    let total = p.rows * p.cols;
    let botones = (1..=total)
        .map(|i| match p.botones.iter().find(|b| b.index == i) {
            Some(b) => button_to_lfa(i, b),
            None => empty_button(i),
        })
        .collect();
    LfaPaleta {
        nombre: p.nombre.clone(),
        rows: p.rows,
        cols: p.cols,
        audio_out: if p.audio_out.is_empty() {
            default_out()
        } else {
            p.audio_out.clone()
        },
        shortcut: p.shortcut.clone(),
        midi: p.midi.clone(),
        tab_bg: p.tab_bg.clone(),
        tab_text: p.tab_text.clone(),
        botones,
    }
}

fn button_to_lfa(id: u32, b: &ButtonData) -> LfaButton {
    LfaButton {
        id,
        label: b.label.clone(),
        type_field: b.type_field.clone(),
        file: b.path.clone(),
        folder: b.folder.clone(),
        name: b.name.clone(),
        bg: b.color_bg.clone(),
        text: b.color_text.clone(),
        vol: b.vol,
        loop_mode: b.loop_mode,
        stop_other: b.stop_other,
        overlap: b.overlap,
        restart: b.restart,
        shortcut: b.shortcut.clone(),
        midi: b.midi.clone(),
        visual: b.visual.clone(),
    }
}

fn empty_button(id: u32) -> LfaButton {
    LfaButton {
        id,
        label: id.to_string(),
        type_field: default_type(),
        file: String::new(),
        folder: String::new(),
        name: String::new(),
        bg: String::new(),
        text: default_text(),
        vol: 1.0,
        loop_mode: false,
        stop_other: false,
        overlap: false,
        restart: false,
        shortcut: String::new(),
        midi: Default::default(),
        visual: Default::default(),
    }
}

/// Importa solo las celdas con contenido (archivo o tipo especial).
/// El indice es el id numerico; si falta el color se asigna uno aleatorio.
pub fn from_lfa_paleta(p: LfaPaleta, id: String) -> PaletaData {
    let botones = p
        .botones
        .into_iter()
        .filter(|b| !b.file.is_empty() || b.type_field != "audio")
        .map(|b| button_from_lfa(b, &id))
        .collect();
    PaletaData {
        id,
        nombre: p.nombre,
        rows: p.rows,
        cols: p.cols,
        audio_out: if p.audio_out == "global" {
            String::new()
        } else {
            p.audio_out
        },
        shortcut: p.shortcut,
        midi: p.midi,
        tab_bg: p.tab_bg,
        tab_text: p.tab_text,
        botones,
    }
}

fn button_from_lfa(b: LfaButton, paleta_id: &str) -> ButtonData {
    ButtonData {
        id: format!("{}_btn_{}", paleta_id, b.id),
        index: b.id,
        label: b.label,
        type_field: b.type_field,
        path: b.file,
        folder: b.folder,
        name: b.name,
        color_bg: if b.bg.is_empty() {
            random_color()
        } else {
            b.bg
        },
        color_text: b.text,
        vol: b.vol,
        duration: 0.0,
        duration_str: String::new(),
        loop_mode: b.loop_mode,
        stop_other: b.stop_other,
        overlap: b.overlap,
        restart: b.restart,
        shortcut: b.shortcut,
        midi: b.midi,
        visual: b.visual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::button::defaults::new_button;
    use crate::model::ButtonVisual;

    #[test]
    fn visual_survives_the_shared_tab_format() {
        let mut button = new_button("source", 1, "Aplausos", "#000000", "#ffffff");
        button.path = "aplausos.wav".into();
        button.visual = ButtonVisual {
            kind: "emoji".into(),
            value: "👏".into(),
            mode: "visual_text".into(),
        };
        let source = PaletaData {
            id: "source".into(),
            nombre: "Efectos".into(),
            rows: 1,
            cols: 1,
            audio_out: String::new(),
            shortcut: String::new(),
            midi: Default::default(),
            tab_bg: String::new(),
            tab_text: String::new(),
            botones: vec![button],
        };

        let json = serde_json::to_string(&to_lfa_paleta(&source)).unwrap();
        let encoded: LfaPaleta = serde_json::from_str(&json).unwrap();
        let restored = from_lfa_paleta(encoded, "target".into());
        assert_eq!(restored.botones[0].visual.value, "👏");
        assert_eq!(restored.botones[0].visual.mode, "visual_text");
    }
}
