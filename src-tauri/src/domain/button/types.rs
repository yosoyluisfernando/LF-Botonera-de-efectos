/// Modulo: button_types.rs
/// Proposito: fuente unica de tipos de boton y visibilidad por configuracion.
use crate::model::AppConfig;
use serde::Serialize;

#[derive(Serialize)]
pub struct ButtonTypeOption {
    pub id: &'static str,
    pub label_key: &'static str,
    pub placeholder_key: &'static str,
    pub is_folder: bool,
    pub is_locution: bool,
    pub can_prelisten: bool,
    pub default_folder: String,
}

#[derive(Serialize)]
pub struct ButtonTypeState {
    pub selected_type: String,
    pub options: Vec<ButtonTypeOption>,
}

/// Devuelve los tipos que el editor puede mostrar en este momento.
pub fn editor_state(cfg: &AppConfig, current_type: Option<&str>) -> ButtonTypeState {
    let options = available_options(cfg);
    let selected_type = current_type
        .filter(|kind| options.iter().any(|opt| opt.id == *kind))
        .unwrap_or("audio")
        .to_string();
    ButtonTypeState {
        selected_type,
        options,
    }
}

/// Valida que un tipo exista y este habilitado segun la configuracion global.
pub fn validate_enabled(cfg: &AppConfig, btn_type: &str) -> Result<(), String> {
    match btn_type {
        "time" if !time_enabled(cfg) => Err("time_disabled".to_string()),
        "temperature" | "humidity" if !weather_enabled(cfg) => Err("weather_disabled".to_string()),
        kind if available_options(cfg).iter().any(|opt| opt.id == kind) => Ok(()),
        _ => Err("invalid_button_type".to_string()),
    }
}

fn available_options(cfg: &AppConfig) -> Vec<ButtonTypeOption> {
    let mut options = vec![
        option(
            "audio",
            "type_audio",
            "placeholder_audio",
            false,
            false,
            true,
            "",
        ),
        option(
            "random_folder",
            "type_random_folder",
            "placeholder_random_folder",
            true,
            false,
            false,
            "",
        ),
    ];
    if time_enabled(cfg) {
        options.push(option(
            "time",
            "type_time",
            "placeholder_time",
            true,
            true,
            false,
            &cfg.locutions.time_folder,
        ));
    }
    if weather_enabled(cfg) {
        options.push(option(
            "temperature",
            "type_temp",
            "placeholder_temp",
            true,
            true,
            false,
            &cfg.locutions.temp_folder,
        ));
        options.push(option(
            "humidity",
            "type_hum",
            "placeholder_hum",
            true,
            true,
            false,
            &cfg.locutions.hum_folder,
        ));
    }
    options
}

fn option(
    id: &'static str,
    label: &'static str,
    placeholder: &'static str,
    is_folder: bool,
    is_locution: bool,
    can_prelisten: bool,
    default_folder: &str,
) -> ButtonTypeOption {
    ButtonTypeOption {
        id,
        label_key: edit_key(label),
        placeholder_key: edit_key(placeholder),
        is_folder,
        is_locution,
        can_prelisten,
        default_folder: default_folder.to_string(),
    }
}

fn edit_key(key: &'static str) -> &'static str {
    match key {
        "type_audio" => "edit_modal.type_audio",
        "type_random_folder" => "edit_modal.type_random_folder",
        "type_time" => "edit_modal.type_time",
        "type_temp" => "edit_modal.type_temp",
        "type_hum" => "edit_modal.type_hum",
        "placeholder_audio" => "edit_modal.placeholder_audio",
        "placeholder_random_folder" => "edit_modal.placeholder_random_folder",
        "placeholder_time" => "edit_modal.placeholder_time",
        "placeholder_temp" => "edit_modal.placeholder_temp",
        "placeholder_hum" => "edit_modal.placeholder_hum",
        _ => "edit_modal.type_audio",
    }
}

fn time_enabled(cfg: &AppConfig) -> bool {
    cfg.weather_module_enabled && cfg.locutions.time_enabled
}

fn weather_enabled(cfg: &AppConfig) -> bool {
    cfg.weather_module_enabled && cfg.locutions.weather_enabled
}
