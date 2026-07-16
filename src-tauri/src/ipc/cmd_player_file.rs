//! Modulo: cmd_player_file.rs
//! Proposito: guardar y abrir listas del reproductor en formato `.LFPlay`,
//! compatible con LF Automatizador (array JSON de filas). Reutiliza el adaptador
//! `domain::export::lfa_format` y el patron de dialogos de `cmd_export.rs`.
//! Cancelar el dialogo NO es un error: devuelve `false`/`None` para que la UI
//! pueda abortar sin ruido (p. ej. "guardar antes de limpiar" cancelado).
use super::cmd_player::{player_view, PlayerView};
use super::cmd_player_queue::sync_queue;
use super::AppState;
use crate::domain::export::lfa_format::{from_lfa_row, to_lfa_row, LfaPlaylistRow};
use crate::engine::persist::config_io;
use serde_json::Value;

const FILTER: &str = "LF Automatizador Playlist";
const EXT: &str = "LFPlay";

/// Guarda la cola actual. `false` = el usuario cerro el dialogo.
#[tauri::command]
pub fn player_save_playlist(state: tauri::State<AppState>) -> Result<bool, String> {
    let rows: Vec<LfaPlaylistRow> = {
        let cfg = state.config.lock().unwrap();
        cfg.player.tracks.iter().map(to_lfa_row).collect()
    };
    if rows.is_empty() {
        return Err("player_empty_queue".into());
    }
    let Some(path) = rfd::FileDialog::new()
        .add_filter(FILTER, &[EXT])
        .set_file_name(&format!("playlist.{EXT}"))
        .save_file()
    else {
        return Ok(false);
    };
    let json = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Abre una lista y REEMPLAZA la cola actual. `None` = el usuario cancelo.
#[tauri::command]
pub fn player_open_playlist(state: tauri::State<AppState>) -> Result<Option<PlayerView>, String> {
    let Some(path) = rfd::FileDialog::new().add_filter(FILTER, &[EXT]).pick_file() else {
        return Ok(None);
    };
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let rows: Vec<LfaPlaylistRow> = serde_json::from_value(extract_rows(value))
        .map_err(|_| "player_invalid_playlist".to_string())?;
    {
        let mut cfg = state.config.lock().unwrap();
        let theme = cfg.theme.clone();
        let mut tracks = Vec::new();
        for row in &rows {
            let bg = crate::domain::colors::random_color();
            let text = crate::domain::colors::text_for_theme(&bg, &theme, "button");
            if let Some(btn) = from_lfa_row(row, tracks.len() as u32 + 1, &bg, &text) {
                tracks.push(btn);
            }
        }
        if tracks.is_empty() {
            return Err("player_invalid_playlist".into());
        }
        cfg.player.tracks = tracks;
        config_io::save_config(&cfg)?;
    }
    sync_queue(&state);
    Ok(Some(player_view(&state)))
}

/// Acepta el array suelto del Automatizador o un objeto que lo envuelva.
fn extract_rows(value: Value) -> Value {
    if value.is_array() {
        return value;
    }
    for key in ["rows", "botones", "tracks"] {
        if let Some(found) = value.get(key) {
            if found.is_array() {
                return found.clone();
            }
        }
    }
    value
}
