//! Modulo: domain/grid/search.rs
//! Proposito: el buscador global de efectos. Encuentra botones cargados en
//! CUALQUIER pestaña del perfil por su texto (etiqueta, nombre, archivo,
//! carpeta) o por su ubicacion (nombre de la pestaña y posicion).
//!
//! Pura y barata: recorre la configuracion en memoria (cientos de botones son
//! microsegundos) y no toca el audio, asi que buscar nunca interrumpe lo que
//! suena. Ignora tildes y mayusculas con la misma normalizacion que la
//! Biblioteca (`domain/search_text.rs`).
//!
//! Reglas de la consulta (las palabras se combinan con Y):
//! - Una palabra debe aparecer en el texto del boton o en el nombre de su
//!   pestaña: "aplausos", "cortinas aplausos".
//! - Un numero suelto tambien vale como POSICION: "5" encuentra el boton 5 de
//!   cada pestaña; "cortinas 5", el boton 5 de la pestaña Cortinas.
use crate::domain::search_text::normalize;
use crate::model::ProfileData;
use serde::Serialize;
use std::path::Path;

pub const MAX_RESULTS: usize = 60;

#[derive(Debug, Serialize, PartialEq)]
pub struct ButtonHit {
    pub paleta_id: String,
    pub paleta_nombre: String,
    /// Posicion de la pestaña en la barra (1-based), para mostrarla.
    pub tab_position: usize,
    pub button_id: String,
    pub index: u32,
    pub row: u32,
    pub col: u32,
    pub label: String,
    pub file_name: String,
}

pub fn search(profile: &ProfileData, query: &str) -> Vec<ButtonHit> {
    let words: Vec<String> = normalize(query)
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(u8, ButtonHit)> = Vec::new();
    for (tab_i, paleta) in profile.paletas.iter().enumerate() {
        let tab_text = normalize(&paleta.nombre);
        for button in &paleta.botones {
            if button.path.is_empty() && button.type_field == "audio" {
                continue;
            }
            let file_name = file_name(&button.path);
            let text = normalize(&format!(
                "{} {} {} {}",
                button.label,
                button.name,
                file_name,
                folder_name(&button.folder)
            ));
            let Some(rank) = rank(&words, &text, &tab_text, button.index) else {
                continue;
            };
            let cols = paleta.cols.max(1);
            hits.push((
                rank,
                ButtonHit {
                    paleta_id: paleta.id.clone(),
                    paleta_nombre: paleta.nombre.clone(),
                    tab_position: tab_i + 1,
                    button_id: button.id.clone(),
                    index: button.index,
                    row: (button.index - 1) / cols + 1,
                    col: (button.index - 1) % cols + 1,
                    label: button.label.clone(),
                    file_name,
                },
            ));
        }
    }
    // Primero lo que mejor coincide; a igualdad, en el orden de las pestañas y
    // de la rejilla (lo que el operador ve de izquierda a derecha).
    hits.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.tab_position.cmp(&b.1.tab_position))
            .then(a.1.index.cmp(&b.1.index))
    });
    hits.into_iter().take(MAX_RESULTS).map(|(_, h)| h).collect()
}

/// None = no coincide. Menor = mejor: 0 el texto empieza por la consulta,
/// 1 contiene todas las palabras, 2 coincide por ubicacion.
fn rank(words: &[String], text: &str, tab: &str, index: u32) -> Option<u8> {
    let mut by_location = false;
    for word in words {
        if text.contains(word.as_str()) {
            continue;
        }
        if tab.contains(word.as_str()) {
            by_location = true;
            continue;
        }
        if word.parse::<u32>().ok() == Some(index) {
            by_location = true;
            continue;
        }
        return None;
    }
    if by_location {
        Some(2)
    } else if text.starts_with(words[0].as_str()) {
        Some(0)
    } else {
        Some(1)
    }
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn folder_name(folder: &str) -> String {
    Path::new(folder)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod search_tests;
