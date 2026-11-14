use crate::model::grid::{ButtonView, GridState};
/// Modulo: grid_view.rs
/// Proposito: preparar la vista de grilla que consume el frontend.
use crate::model::{ButtonData, PaletaData};

pub fn paleta_to_grid(paleta: &PaletaData) -> GridState {
    GridState {
        obs: paleta.obs,
        columns: paleta.cols,
        rows: paleta.rows,
        buttons: paleta.botones.iter().map(button_to_view).collect(),
    }
}

pub fn button_to_view(button: &ButtonData) -> ButtonView {
    let mut data = button.clone();
    if data.type_field == "audio" && !data.path.is_empty() {
        let converted = crate::engine::audio::transcode::resolve(&data.path)
            .is_ok_and(|resolved| resolved != data.path);
        if converted {
            let duration = crate::engine::audio::formats::probe_duration_secs(&data.path);
            if duration > 0.0 {
                data.duration = duration;
                data.duration_str = format!("{duration:.1}s");
            }
        }
    }
    ButtonView {
        data,
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
