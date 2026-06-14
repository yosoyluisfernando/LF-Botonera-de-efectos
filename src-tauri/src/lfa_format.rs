/// Módulo: lfa_format.rs
/// Propósito: Estructuras y conversiones del formato .bdelf/.bdeplf, calcadas
/// del normalizador del LF Automatizador (backend/ipc/cartwall.js). Regla 5:
/// lo que se guarda aquí se abre allá sin perder nada, y viceversa.
///  - Botones: por POSICIÓN del array (posición i = celda i+1), id numérico.
///  - Se exporta la cuadrícula completa (celdas vacías incluidas).
///  - audioOut: el LFA usa el literal "global"; internamente usamos "".
///  - El perfil lleva config { outMain, outPre, keys { stopAll, next, prev } }.

use crate::colors::random_color;
use crate::types::{AudioConfig, ButtonData, PaletaData, ProfileData};
use serde::{Deserialize, Serialize};

// ─── Structs (espejo del normalizador del LFA) ───────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct LfaButton {
    pub id: u32,
    #[serde(default)] pub label: String,
    #[serde(default = "default_type", rename = "type")]
    pub type_field: String,
    #[serde(default)] pub file: String,
    #[serde(default)] pub folder: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub bg: String,
    #[serde(default = "default_text")] pub text: String,
    #[serde(default = "default_vol")] pub vol: f32,
    #[serde(default, rename = "loop")]      pub loop_mode: bool,
    #[serde(default, rename = "stopOther")] pub stop_other: bool,
    #[serde(default)] pub overlap: bool,
    #[serde(default)] pub restart: bool,
    #[serde(default)] pub shortcut: String,
}

fn default_type() -> String { "audio".to_string() }
fn default_text() -> String { "#FFFFFF".to_string() }
fn default_vol()  -> f32    { 1.0 }

#[derive(Serialize, Deserialize)]
pub struct LfaPaleta {
    pub nombre: String,
    #[serde(default = "default_dim")] pub rows: u32,
    #[serde(default = "default_dim")] pub cols: u32,
    #[serde(default = "default_out", rename = "audioOut")] pub audio_out: String,
    #[serde(default)] pub shortcut: String,
    #[serde(default, rename = "tabBg")]   pub tab_bg: String,
    #[serde(default, rename = "tabText")] pub tab_text: String,
    pub botones: Vec<LfaButton>,
}

fn default_dim() -> u32    { 5 }
fn default_out() -> String { "global".to_string() }

#[derive(Serialize, Deserialize, Default)]
pub struct LfaKeys {
    #[serde(default, rename = "stopAll")] pub stop_all: String,
    #[serde(default)] pub next: String,
    #[serde(default)] pub prev: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LfaConfig {
    #[serde(default, rename = "outMain")] pub out_main: String,
    #[serde(default, rename = "outPre")]  pub out_pre:  String,
    #[serde(default)] pub keys: LfaKeys,
}

#[derive(Serialize, Deserialize)]
pub struct LfaProfile {
    #[serde(default)] pub id: String,
    pub name: String,
    #[serde(default)] pub bg: String,
    #[serde(default)] pub text: String,
    #[serde(default)] pub config: LfaConfig,
    pub paletas: Vec<LfaPaleta>,
}

// ─── Conversiones ─────────────────────────────────────────────────────────────

/// Exporta la cuadrícula completa: una entrada por celda, vacías incluidas
/// (el LFA empareja botones por posición del array, no por id).
pub fn to_lfa_paleta(p: &PaletaData) -> LfaPaleta {
    let total = p.rows * p.cols;
    let botones = (1..=total).map(|i| {
        match p.botones.iter().find(|b| b.index == i) {
            Some(b) => LfaButton {
                id: i, label: b.label.clone(), type_field: b.type_field.clone(),
                file: b.path.clone(), folder: b.folder.clone(), name: b.name.clone(),
                bg: b.color_bg.clone(), text: b.color_text.clone(), vol: b.vol,
                loop_mode: b.loop_mode, stop_other: b.stop_other,
                overlap: b.overlap, restart: b.restart, shortcut: b.shortcut.clone(),
            },
            None => LfaButton {
                id: i, label: i.to_string(), type_field: default_type(),
                file: String::new(), folder: String::new(), name: String::new(),
                bg: String::new(), text: default_text(), vol: 1.0,
                loop_mode: false, stop_other: false,
                overlap: false, restart: false, shortcut: String::new(),
            },
        }
    }).collect();
    LfaPaleta {
        nombre: p.nombre.clone(), rows: p.rows, cols: p.cols,
        audio_out: if p.audio_out.is_empty() { default_out() } else { p.audio_out.clone() },
        shortcut: p.shortcut.clone(),
        tab_bg: p.tab_bg.clone(), tab_text: p.tab_text.clone(), botones,
    }
}

/// Importa solo las celdas con contenido (archivo o tipo especial).
/// El índice es el id numérico; si falta el color se asigna uno aleatorio.
pub fn from_lfa_paleta(p: LfaPaleta, id: String) -> PaletaData {
    let botones = p.botones.into_iter()
        .filter(|b| !b.file.is_empty() || b.type_field != "audio")
        .map(|b| ButtonData {
            id: format!("{}_btn_{}", id, b.id), index: b.id,
            label: b.label, type_field: b.type_field,
            path: b.file, folder: b.folder, name: b.name,
            color_bg: if b.bg.is_empty() { random_color() } else { b.bg },
            color_text: b.text, vol: b.vol,
            duration: 0.0, duration_str: String::new(),
            loop_mode: b.loop_mode, stop_other: b.stop_other,
            overlap: b.overlap, restart: b.restart, shortcut: b.shortcut,
        })
        .collect();
    PaletaData {
        id, nombre: p.nombre, rows: p.rows, cols: p.cols,
        audio_out: if p.audio_out == "global" { String::new() } else { p.audio_out },
        shortcut: p.shortcut,
        tab_bg: p.tab_bg, tab_text: p.tab_text, botones,
    }
}

/// Perfil completo → formato LFA, incluyendo config (salidas y atajos globales).
pub fn to_lfa_profile(p: &ProfileData) -> LfaProfile {
    LfaProfile {
        id: p.id.clone(), name: p.name.clone(),
        bg: p.bg.clone(), text: p.text.clone(),
        config: LfaConfig {
            out_main: p.audio.out_main.clone(),
            out_pre:  p.audio.out_pre.clone(),
            keys: LfaKeys {
                stop_all: p.audio.key_stop.clone(),
                next:     p.audio.key_next.clone(),
                prev:     p.audio.key_prev.clone(),
            },
        },
        paletas: p.paletas.iter().map(to_lfa_paleta).collect(),
    }
}

/// Formato LFA → perfil interno (config incluida). `new_id` lo decide el caller.
pub fn from_lfa_profile(lfa: LfaProfile, new_id: String) -> ProfileData {
    let paletas: Vec<PaletaData> = lfa.paletas.into_iter().enumerate()
        .map(|(i, p)| from_lfa_paleta(p, format!("{}_paleta_{}", new_id, i)))
        .collect();
    let first_pid = paletas.first().map(|p| p.id.clone()).unwrap_or_default();
    ProfileData {
        id: new_id, name: lfa.name, bg: lfa.bg, text: lfa.text,
        audio: AudioConfig {
            out_main: if lfa.config.out_main.is_empty() { "default".to_string() }
                      else { lfa.config.out_main },
            out_pre:  lfa.config.out_pre,
            global_keys: false,
            key_stop: lfa.config.keys.stop_all,
            key_next: lfa.config.keys.next,
            key_prev: lfa.config.keys.prev,
            playback_mode: "normal".to_string(),
        },
        active_paleta_id: first_pid,
        paletas,
    }
}
