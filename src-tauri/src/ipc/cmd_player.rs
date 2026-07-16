//! Modulo: cmd_player.rs
//! Proposito: IPC de transporte y estado del reproductor auxiliar. Endpoints
//! finos que delegan en el motor propio (`engine/player`). Aqui solo va lo que
//! NO se persiste (es estado de ejecucion); los ajustes que se guardan viven en
//! `cmd_player_config.rs` y la edicion de la cola en `cmd_player_queue.rs`.
//! Los indices son POSICIONES 0-based en la cola.
use super::cmd_player_queue::sync_queue;
use super::AppState;
use crate::domain::grid::view::button_to_view;
use crate::domain::player::PlayerMode;
use crate::engine::player::PlayerSnapshot;
use crate::model::ButtonView;
use serde::Serialize;

/// Vista que consume la UI: cola + ajustes + estado en vivo del motor.
#[derive(Serialize)]
pub struct PlayerView {
    pub tracks: Vec<ButtonView>,
    pub mode: String,
    pub volume: f32,
    pub output_device: String,
    /// Total acumulado en segundos. Lo suma Rust (regla 4): que cuenta y que no
    /// es una regla de negocio, no una decision de la UI.
    pub total_s: f64,
    /// "elapsed" | "remaining": que ensena el contador de la pista.
    pub time_display: String,
    /// "ask" | "always" | "never" al soltar una carpeta grande.
    pub large_folder_action: String,
    pub snapshot: PlayerSnapshot,
}

/// Suma solo lo que tiene duracion CONOCIDA. Los tipos especiales (carpeta
/// aleatoria, hora, clima) no se resuelven hasta sonar, asi que no cuentan:
/// mismo criterio que el Automatizador, que los muestra como `--:--`.
fn queue_total_s(tracks: &[crate::model::ButtonData]) -> f64 {
    tracks
        .iter()
        .filter(|t| t.type_field == "audio")
        .map(|t| t.duration.max(0.0))
        .sum()
}

pub(crate) fn player_view(state: &AppState) -> PlayerView {
    let snapshot = state.player.lock().unwrap().snapshot();
    let cfg = state.config.lock().unwrap();
    PlayerView {
        tracks: cfg.player.tracks.iter().map(button_to_view).collect(),
        mode: cfg.player.playback_mode.clone(),
        volume: cfg.player.volume,
        output_device: cfg.player.output_device.clone(),
        total_s: queue_total_s(&cfg.player.tracks),
        time_display: cfg.player.time_display.clone(),
        large_folder_action: cfg.player.large_folder_action.clone(),
        snapshot,
    }
}

#[tauri::command]
pub fn get_player(state: tauri::State<AppState>) -> PlayerView {
    player_view(&state)
}

/// Estado en vivo ligero: lo sondea la UI en cada tick para pintar la pista que
/// suena (verde), la marcada como siguiente (naranja) y el tiempo. Sin la cola.
#[tauri::command]
pub fn get_player_snapshot(state: tauri::State<AppState>) -> PlayerSnapshot {
    state.player.lock().unwrap().snapshot()
}

#[tauri::command]
pub fn player_play_index(index: u32, state: tauri::State<AppState>) {
    state.player.lock().unwrap().play_index(index as usize);
}

/// Doble clic en una fila. El MOTOR decide: si esta detenido la reproduce; si
/// algo suena la marca como siguiente, sin cortar la musica. La UI no decide.
#[tauri::command]
pub fn player_activate_index(index: u32, state: tauri::State<AppState>) {
    state.player.lock().unwrap().activate_index(index as usize);
}

#[tauri::command]
pub fn player_next(state: tauri::State<AppState>) {
    state.player.lock().unwrap().next();
}
#[tauri::command]
pub fn player_prev(state: tauri::State<AppState>) {
    state.player.lock().unwrap().prev();
}
#[tauri::command]
pub fn player_stop(state: tauri::State<AppState>) {
    state.player.lock().unwrap().stop();
}
#[tauri::command]
pub fn player_pause(state: tauri::State<AppState>) {
    state.player.lock().unwrap().pause();
}
#[tauri::command]
pub fn player_resume(state: tauri::State<AppState>) {
    state.player.lock().unwrap().resume();
}

/// Marca (o desmarca con `None`) la pista siguiente. Es ley: siempre se respeta.
#[tauri::command]
pub fn player_mark_next(index: Option<u32>, state: tauri::State<AppState>) {
    state.player.lock().unwrap().mark_next(index.map(|i| i as usize));
}

/// Detener al finalizar: al terminar la pista actual, la siguiente NO arranca
/// sola hasta pulsar play; lo marcado como siguiente se conserva.
#[tauri::command]
pub fn player_set_stop_after(enabled: bool, state: tauri::State<AppState>) {
    state.player.lock().unwrap().set_stop_after(enabled);
}

/// Boton Loop: repite la cancion actual hasta desactivarlo. No confundir con el
/// modo `repeat`, que repite la LISTA entera. Es de transporte: no se persiste.
#[tauri::command]
pub fn player_set_loop(enabled: bool, state: tauri::State<AppState>) {
    state.player.lock().unwrap().set_loop_current(enabled);
}

/// Salta a una posicion de lo que suena (segundos desde el cue de inicio). El
/// motor lo ignora si la pista no es reposicionable (`can_seek` del snapshot).
#[tauri::command]
pub fn player_seek(position_s: f64, state: tauri::State<AppState>) -> Result<(), String> {
    if !position_s.is_finite() || position_s < 0.0 {
        return Err("invalid_position".into());
    }
    state.player.lock().unwrap().seek(position_s);
    Ok(())
}

/// Arranque: sincroniza modo y cola con el motor tras aplicar dispositivo/volumen.
pub(crate) fn apply_startup(state: &AppState) {
    let mode = PlayerMode::from_config(&state.config.lock().unwrap().player.playback_mode);
    state.player.lock().unwrap().set_mode(mode);
    sync_queue(state);
}
