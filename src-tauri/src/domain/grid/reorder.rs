/// Modulo: grid_reorder.rs
/// Proposito: reordenar botones de la grilla y trasladar su estado de audio
/// activo cuando cambian de posicion.
use crate::ipc::cmd_grid::{active_paleta, save_grid};
use crate::domain::playback::state as playback_state;
use crate::model::grid::GridState;
use crate::model::ButtonData;
use crate::AppState;

#[tauri::command]
pub fn reorder_buttons(
    from_index: u32,
    to_index: u32,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let should_remember = {
        let paleta = active_paleta(&mut cfg)?;
        paleta.botones.iter().any(|b| b.index == from_index) && from_index != to_index
    };
    if should_remember {
        state.history.lock().unwrap().remember(&cfg);
    }
    let paleta = active_paleta(&mut cfg)?;
    let pid = paleta.id.clone();
    let pos_src = paleta.botones.iter().position(|b| b.index == from_index);
    let pos_dst = paleta.botones.iter().position(|b| b.index == to_index);
    let mut id_mappings = Vec::new();

    match (pos_src, pos_dst) {
        (Some(a), Some(b)) => {
            push_id_mapping(&mut id_mappings, &paleta.botones[a], &pid, to_index);
            push_id_mapping(&mut id_mappings, &paleta.botones[b], &pid, from_index);
            move_button(&mut paleta.botones[a], &pid, to_index);
            move_button(&mut paleta.botones[b], &pid, from_index);
        }
        (Some(a), None) => {
            push_id_mapping(&mut id_mappings, &paleta.botones[a], &pid, to_index);
            move_button(&mut paleta.botones[a], &pid, to_index);
        }
        _ => {}
    }

    let grid = save_grid(&mut cfg)?;
    drop(cfg);
    remap_active_audio_ids(&state, &id_mappings);
    Ok(grid)
}

fn push_id_mapping(
    mappings: &mut Vec<(String, String)>,
    btn: &ButtonData,
    paleta_id: &str,
    next_index: u32,
) {
    let next_id = button_id(paleta_id, next_index);
    if btn.id != next_id {
        mappings.push((btn.id.clone(), next_id));
    }
}

fn move_button(btn: &mut ButtonData, paleta_id: &str, index: u32) {
    btn.index = index;
    btn.id = button_id(paleta_id, index);
}

fn button_id(paleta_id: &str, index: u32) -> String {
    format!("{}_btn_{}", paleta_id, index)
}

pub(crate) fn remap_active_audio_ids(
    state: &tauri::State<AppState>,
    mappings: &[(String, String)],
) {
    if mappings.is_empty() {
        return;
    }
    let engine = state.audio.lock().unwrap();
    playback_state::remap_button_ids(
        engine.button_states_handle(),
        engine.last_pressed_handle(),
        mappings,
    );
}
