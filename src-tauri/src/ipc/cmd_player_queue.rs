//! Modulo: cmd_player_queue.rs
//! Proposito: IPC de edicion de la cola del reproductor auxiliar (anadir desde el
//! explorador, anadir un boton arrastrado, quitar, reordenar, vaciar) y la
//! sincronizacion de la cola resuelta con el motor. Reutiliza `resolve_edit`
//! (cue/gain) y los helpers de botones ya existentes. Indices = POSICION 0-based.
use super::cmd_player::{player_view, PlayerView};
use crate::domain::playback::edit::resolve_edit;
use super::AppState;
use crate::domain::button::defaults::new_button;
use crate::engine::audio::formats::{probe_duration_secs, validate_audio_file};
use crate::engine::persist::config_io;
use crate::engine::player::QueueEntry;
use crate::model::{AppConfig, ButtonData};

/// Construye la cola desde la config. Solo se resuelve aqui el audio normal; los
/// tipos especiales (carpeta aleatoria, hora, clima) viajan con su `kind` y su
/// carpeta y los resuelve el motor AL SONAR, porque la hora avanza, el clima
/// cambia y el aleatorio debe dar una cancion distinta en cada pasada.
pub(crate) fn build_entries(state: &AppState) -> Vec<QueueEntry> {
    let tracks = state.config.lock().unwrap().player.tracks.clone();
    tracks.iter().map(|track| entry_for(state, track)).collect()
}

fn entry_for(state: &AppState, btn: &ButtonData) -> QueueEntry {
    let base = QueueEntry {
        id: btn.id.clone(),
        kind: btn.type_field.clone(),
        folder: btn.folder.clone(),
        gain: btn.vol.max(0.0),
        duration_s: btn.duration.max(0.0),
        ..Default::default()
    };
    if btn.type_field != "audio" {
        return base;
    }
    if btn.path.is_empty() || validate_audio_file(&btn.path).is_err() {
        return QueueEntry { id: btn.id.clone(), ..Default::default() };
    }
    let edit = resolve_edit(&state.tracks, &btn.path, btn.duration);
    QueueEntry {
        path: btn.path.clone(),
        cue_start_s: edit.cue_start_s,
        cue_end_s: edit.cue_end_s,
        gain: edit.file_gain * btn.vol.max(0.0),
        duration_s: edit.duration,
        ..base
    }
}

/// Empuja la cola resuelta al motor. Se llama tras cada edicion y al arrancar.
pub(crate) fn sync_queue(state: &AppState) {
    let entries = build_entries(state);
    state.player.lock().unwrap().set_queue(entries);
}

/// Renumera solo la posicion visible. El id permanece estable para que el motor
/// pueda conservar current/next al reordenar o quitar otras filas.
fn reindex(tracks: &mut [ButtonData]) {
    for (position, track) in tracks.iter_mut().enumerate() {
        track.index = position as u32 + 1;
    }
}

/// Inserta en la posicion indicada (o al final si no se envia) y renumera.
pub(super) fn insert_track(tracks: &mut Vec<ButtonData>, btn: ButtonData, index: Option<u32>) {
    let position = index.map_or(tracks.len(), |i| (i as usize).min(tracks.len()));
    tracks.insert(position, btn);
    reindex(tracks);
}

pub(super) fn next_id(tracks: &[ButtonData]) -> String {
    (1..)
        .map(|n| format!("player_btn_{n}"))
        .find(|id| tracks.iter().all(|track| track.id != *id))
        .expect("la cola no puede agotar usize")
}

/// Anade una pista del explorador. `index` = posicion donde insertarla; sin ella,
/// va al final (soltar en el espacio vacio de la lista).
#[tauri::command]
pub fn player_add_track(
    path: String,
    index: Option<u32>,
    state: tauri::State<AppState>,
) -> Result<PlayerView, String> {
    if path.is_empty() {
        return Err("player_empty_path".into());
    }
    validate_audio_file(&path)?;
    let name = std::path::Path::new(&path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    {
        let mut cfg = state.config.lock().unwrap();
        let bg = crate::domain::colors::random_color();
        let text = crate::domain::colors::text_for_theme(&bg, &cfg.theme, "button");
        let mut btn = new_button("player", 1, &name, &bg, &text);
        btn.id = next_id(&cfg.player.tracks);
        btn.duration = probe_duration_secs(&path);
        btn.duration_str = if btn.duration > 0.0 {
            format!("{:.1}s", btn.duration)
        } else {
            String::new()
        };
        btn.path = path;
        insert_track(&mut cfg.player.tracks, btn, index);
        config_io::save_config(&cfg)?;
    }
    sync_queue(&state);
    Ok(player_view(&state))
}

/// Anade una copia de un boton existente (arrastrado desde la botonera principal
/// o el panel fijo). `index` = posicion; sin ella, al final.
#[tauri::command]
pub fn player_add_button(
    button_id: String,
    index: Option<u32>,
    state: tauri::State<AppState>,
) -> Result<PlayerView, String> {
    let mut btn = find_button(&state.config.lock().unwrap(), &button_id).ok_or("button_not_found")?;
    {
        let mut cfg = state.config.lock().unwrap();
        btn.id = next_id(&cfg.player.tracks);
        insert_track(&mut cfg.player.tracks, btn, index);
        config_io::save_config(&cfg)?;
    }
    sync_queue(&state);
    Ok(player_view(&state))
}

fn find_button(cfg: &AppConfig, id: &str) -> Option<ButtonData> {
    let fixed = if cfg.fixed_panel.scope == "profile" {
        cfg.active_profile().map(|p| p.fixed_buttons.as_slice()).unwrap_or(&[])
    } else {
        cfg.fixed_panel.global_buttons.as_slice()
    };
    fixed
        .iter()
        .chain(cfg.profiles.iter().flat_map(|p| &p.paletas).flat_map(|p| &p.botones))
        .find(|b| b.id == id)
        .cloned()
}

#[tauri::command]
pub fn player_remove_track(index: u32, state: tauri::State<AppState>) -> Result<PlayerView, String> {
    {
        let mut cfg = state.config.lock().unwrap();
        let position = index as usize;
        if position >= cfg.player.tracks.len() {
            return Err("button_not_found".into());
        }
        cfg.player.tracks.remove(position);
        reindex(&mut cfg.player.tracks);
        config_io::save_config(&cfg)?;
    }
    sync_queue(&state);
    Ok(player_view(&state))
}

#[tauri::command]
pub fn player_reorder_tracks(from_index: u32, to_index: u32, state: tauri::State<AppState>) -> Result<PlayerView, String> {
    {
        let mut cfg = state.config.lock().unwrap();
        let tracks = &mut cfg.player.tracks;
        let (from, to) = (from_index as usize, to_index as usize);
        if from >= tracks.len() {
            return Err("button_not_found".into());
        }
        let item = tracks.remove(from);
        tracks.insert(to.min(tracks.len()), item);
        reindex(tracks);
        config_io::save_config(&cfg)?;
    }
    sync_queue(&state);
    Ok(player_view(&state))
}

#[tauri::command]
pub fn player_clear_queue(state: tauri::State<AppState>) -> Result<PlayerView, String> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.player.tracks.clear();
        config_io::save_config(&cfg)?;
    }
    sync_queue(&state);
    Ok(player_view(&state))
}
