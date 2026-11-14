/// Modulo: cmd_button_search.rs
/// Proposito: buscador global de efectos (lupa a la izquierda de las pestañas).
/// La regla vive en `domain/grid/search.rs`; aqui solo se lee el perfil activo.
use super::AppState;
use crate::domain::grid::search::{search, ButtonHit};

#[tauri::command]
pub fn search_buttons(query: String, state: tauri::State<AppState>) -> Vec<ButtonHit> {
    let cfg = state.config.lock().unwrap();
    cfg.active_profile()
        .map(|profile| search(profile, &query))
        .unwrap_or_default()
}
