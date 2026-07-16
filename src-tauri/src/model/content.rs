//! Modulo: model/content.rs
//! Proposito: el CONTENIDO que crea el usuario, en su jerarquia natural:
//! perfil → paleta → boton. Son datos puros y serializables; las preferencias de
//! la aplicacion viven aparte, en `model/config.rs` (`AppConfig`).
//!
//! `ButtonData` es el ladrillo comun: lo usan la rejilla, el panel fijo y la cola
//! del reproductor, por eso no cuelga de ningun modulo concreto.
use crate::model::audio::AudioConfig;
use serde::{Deserialize, Serialize};

/// Un boton. Todos los campos nuevos llevan `#[serde(default)]` para que el LF
/// Automatizador pueda leer el archivo ignorandolos (regla 6).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonData {
    pub id: String,
    pub index: u32,
    pub label: String,
    #[serde(default = "default_type", rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub folder: String,
    #[serde(default)]
    pub name: String,
    pub color_bg: String,
    pub color_text: String,
    #[serde(default = "default_vol")]
    pub vol: f32,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub duration_str: String,
    #[serde(default)]
    pub loop_mode: bool,
    #[serde(default)]
    pub stop_other: bool,
    #[serde(default)]
    pub overlap: bool,
    #[serde(default)]
    pub restart: bool,
    #[serde(default)]
    pub shortcut: String,
}
fn default_type() -> String {
    "audio".to_string()
}
fn default_vol() -> f32 {
    1.0
}

/// Una paleta (pestana): su propia rejilla de botones.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaletaData {
    pub id: String,
    pub nombre: String,
    #[serde(default = "default_rows")]
    pub rows: u32,
    #[serde(default = "default_cols")]
    pub cols: u32,
    #[serde(default)]
    pub audio_out: String,
    #[serde(default)]
    pub shortcut: String,
    #[serde(default)]
    pub tab_bg: String,
    #[serde(default)]
    pub tab_text: String,
    pub botones: Vec<ButtonData>,
}
fn default_rows() -> u32 {
    5
}
fn default_cols() -> u32 {
    5
}

/// Un perfil: sus paletas, su salida de audio y sus botones fijos (estos ultimos
/// solo se usan cuando el panel fijo esta en alcance "profile").
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub bg: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub active_paleta_id: String,
    pub paletas: Vec<PaletaData>,
    #[serde(default)]
    pub fixed_buttons: Vec<ButtonData>,
}
