//! Modulo: cmd_fixed_move.rs
//! Proposito: mover botones entre la grilla y el panel fijo (ambos sentidos).
//! Reutiliza la mecanica de intercambio/mover que ya existe para pestanas y
//! reordenamiento: quitar el ButtonData por valor, reescribir index+id con el
//! prefijo del destino, empujarlo al otro lado, remapear el audio activo.
use super::cmd_fixed_panel::{button_prefix, buttons, buttons_mut, ensure_capacity, next_index};
use super::AppState;
use crate::engine::persist::config_io;
use crate::model::{AppConfig, ButtonData};

/// Arrastra un boton de la grilla al panel fijo. Si el hueco fijo destino esta
/// ocupado se intercambian; si `to_fixed_index` es `None`, se anexa al final.
#[tauri::command]
pub fn move_button_to_fixed(
    from_paleta_id: String,
    from_index: u32,
    to_fixed_index: Option<u32>,
    state_: tauri::State<AppState>,
) -> Result<(), String> {
    let mappings = {
        let mut cfg = state_.config.lock().unwrap();
        if !grid_has(&cfg, &from_paleta_id, from_index) {
            return Err("button_not_found".into());
        }
        let fixed_index = to_fixed_index.unwrap_or_else(|| next_index(&cfg));
        if !fixed_has(&cfg, fixed_index) {
            ensure_capacity(&cfg, false)?;
        }
        state_.history.lock().unwrap().remember(&cfg);
        let mappings = relocate(&mut cfg, &from_paleta_id, from_index, fixed_index)?;
        config_io::save_config(&cfg)?;
        mappings
    };
    remap_active_audio(&state_, &mappings);
    Ok(())
}

/// Arrastra un boton fijo a una celda de la grilla. Si la celda esta ocupada se
/// intercambian (el boton de la grilla pasa al panel fijo).
#[tauri::command]
pub fn move_fixed_to_button(
    from_fixed_index: u32,
    to_paleta_id: String,
    to_index: u32,
    state_: tauri::State<AppState>,
) -> Result<(), String> {
    let mappings = {
        let mut cfg = state_.config.lock().unwrap();
        if !fixed_has(&cfg, from_fixed_index) {
            return Err("button_not_found".into());
        }
        state_.history.lock().unwrap().remember(&cfg);
        let mappings = relocate(&mut cfg, &to_paleta_id, to_index, from_fixed_index)?;
        config_io::save_config(&cfg)?;
        mappings
    };
    remap_active_audio(&state_, &mappings);
    Ok(())
}

/// Traslada el estado de audio en curso a los ids nuevos. Se llama con el lock de
/// `config` ya liberado: como el panel principal, nunca se anida otro lock bajo el
/// de `config`. Un traslado no cambia el atajo, asi que no se re-sincroniza el SO.
fn remap_active_audio(state_: &tauri::State<AppState>, mappings: &[(String, String)]) {
    if mappings.is_empty() {
        return;
    }
    let audio = state_.audio.lock().unwrap();
    crate::domain::playback::state::remap_button_ids(
        audio.button_states_handle(),
        audio.last_pressed_handle(),
        mappings,
    );
}

/// Intercambio/mover unificado entre una celda de grilla y un hueco fijo.
/// Devuelve los pares (id_antiguo, id_nuevo) para remapear el audio en curso.
fn relocate(
    cfg: &mut AppConfig,
    paleta_id: &str,
    grid_index: u32,
    fixed_index: u32,
) -> Result<Vec<(String, String)>, String> {
    crate::domain::obs::palette(cfg,paleta_id)?;
    let fixed_prefix = button_prefix(cfg);
    let grid_btn = take_grid_button(cfg, paleta_id, grid_index)?;
    let fixed_btn = take_fixed_button(cfg, fixed_index)?;
    let mut mappings = Vec::new();
    if let Some(mut btn) = grid_btn {
        let new_id = format!("{fixed_prefix}_btn_{fixed_index}");
        mappings.push((btn.id.clone(), new_id.clone()));
        btn.index = fixed_index;
        btn.id = new_id;
        buttons_mut(cfg)?.push(btn);
    }
    if let Some(mut btn) = fixed_btn {
        let new_id = format!("{paleta_id}_btn_{grid_index}");
        mappings.push((btn.id.clone(), new_id.clone()));
        btn.index = grid_index;
        btn.id = new_id;
        push_grid_button(cfg, paleta_id, btn)?;
    }
    Ok(mappings)
}

fn grid_has(cfg: &AppConfig, paleta_id: &str, index: u32) -> bool {
    cfg.active_profile()
        .and_then(|p| p.paletas.iter().find(|pl| pl.id == paleta_id))
        .is_some_and(|pl| pl.botones.iter().any(|b| b.index == index))
}

fn fixed_has(cfg: &AppConfig, index: u32) -> bool {
    buttons(cfg).iter().any(|b| b.index == index)
}

fn take_grid_button(
    cfg: &mut AppConfig,
    paleta_id: &str,
    index: u32,
) -> Result<Option<ButtonData>, String> {
    let paleta = cfg
        .active_profile_mut()
        .ok_or("active_profile_not_found")?
        .paletas
        .iter_mut()
        .find(|p| p.id == paleta_id)
        .ok_or("paleta_not_found")?;
    let pos = paleta.botones.iter().position(|b| b.index == index);
    Ok(pos.map(|p| paleta.botones.remove(p)))
}

fn take_fixed_button(cfg: &mut AppConfig, index: u32) -> Result<Option<ButtonData>, String> {
    let list = buttons_mut(cfg)?;
    let pos = list.iter().position(|b| b.index == index);
    Ok(pos.map(|p| list.remove(p)))
}

fn push_grid_button(cfg: &mut AppConfig, paleta_id: &str, btn: ButtonData) -> Result<(), String> {
    cfg.active_profile_mut()
        .ok_or("active_profile_not_found")?
        .paletas
        .iter_mut()
        .find(|p| p.id == paleta_id)
        .ok_or("paleta_not_found")?
        .botones
        .push(btn);
    Ok(())
}

#[cfg(test)]
#[path = "cmd_fixed_move_tests.rs"]
mod tests;
