use crate::model::grid::{ButtonView, GridState};
/// Modulo: grid_view.rs
/// Proposito: preparar la vista de grilla que consume el frontend.
use crate::model::{ButtonData, PaletaData};

pub fn paleta_to_grid(paleta: &PaletaData) -> GridState {
    GridState {
        columns: paleta.cols,
        rows: paleta.rows,
        buttons: paleta.botones.iter().map(button_to_view).collect(),
    }
}

pub fn button_to_view(button: &ButtonData) -> ButtonView {
    ButtonView {
        data: button.clone(),
        timer_label_key: timer_label_key(&button.type_field).to_string(),
        type_icon: type_icon(&button.type_field).to_string(),
        can_prelisten: can_prelisten(button),
    }
}

fn can_prelisten(button: &ButtonData) -> bool {
    button.type_field == "audio" && !button.path.trim().is_empty()
}

fn type_icon(btn_type: &str) -> &'static str {
    match btn_type {
        "random_folder" => "random_folder",
        "time" => "time",
        "temperature" => "temperature",
        "humidity" => "humidity",
        _ => "",
    }
}

fn timer_label_key(btn_type: &str) -> &'static str {
    match btn_type {
        "random_folder" => "grid.random_folder_badge",
        "time" | "temperature" | "humidity" => "",
        _ => "",
    }
}
