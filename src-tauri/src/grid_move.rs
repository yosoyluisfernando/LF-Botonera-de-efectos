/// Modulo: grid_move.rs
/// Proposito: mover botones entre pestanas sin duplicar datos en la UI.
use super::AppState;
use crate::config;
use crate::grid_reorder;
use crate::types::{AppConfig, ButtonData, PaletaData};

#[tauri::command]
pub fn move_button_to_paleta(
    from_paleta_id: String,
    from_index: u32,
    to_paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    if from_paleta_id == to_paleta_id {
        return Ok(state.config.lock().unwrap().clone());
    }

    let mut cfg = state.config.lock().unwrap();
    let plan = plan_move(&cfg, &from_paleta_id, from_index, &to_paleta_id)?;
    state.history.lock().unwrap().remember(&cfg);
    let (old_id, new_id) = apply_move(&mut cfg, plan)?;
    config::save_config(&cfg)?;
    let next = cfg.clone();
    drop(cfg);
    grid_reorder::remap_active_audio_ids(&state, &[(old_id, new_id)]);
    Ok(next)
}

struct MovePlan {
    profile_pos: usize,
    src_pos: usize,
    dst_pos: usize,
    src_index: u32,
    dst_index: u32,
}

fn plan_move(
    cfg: &AppConfig,
    from_paleta_id: &str,
    from_index: u32,
    to_paleta_id: &str,
) -> Result<MovePlan, String> {
    let profile_pos = cfg
        .profiles
        .iter()
        .position(|p| p.id == cfg.active_profile_id)
        .ok_or("Perfil activo no encontrado")?;
    let profile = &cfg.profiles[profile_pos];
    let src_pos = profile
        .paletas
        .iter()
        .position(|p| p.id == from_paleta_id)
        .ok_or("Pestana origen no encontrada")?;
    let dst_pos = profile
        .paletas
        .iter()
        .position(|p| p.id == to_paleta_id)
        .ok_or("Pestana destino no encontrada")?;
    if !profile.paletas[src_pos]
        .botones
        .iter()
        .any(|b| b.index == from_index)
    {
        return Err("button_not_found".to_string());
    }
    let dst_index = first_empty_index(&profile.paletas[dst_pos])?;
    Ok(MovePlan {
        profile_pos,
        src_pos,
        dst_pos,
        src_index: from_index,
        dst_index,
    })
}

fn apply_move(cfg: &mut AppConfig, plan: MovePlan) -> Result<(String, String), String> {
    let profile = &mut cfg.profiles[plan.profile_pos];
    let mut button = {
        let src_buttons = &mut profile.paletas[plan.src_pos].botones;
        let button_pos = src_buttons
            .iter()
            .position(|b| b.index == plan.src_index)
            .ok_or("button_not_found")?;
        src_buttons.remove(button_pos)
    };
    let old_id = button.id.clone();
    let dst_id = profile.paletas[plan.dst_pos].id.clone();
    move_button(&mut button, &dst_id, plan.dst_index);
    let new_id = button.id.clone();
    profile.paletas[plan.dst_pos].botones.push(button);
    Ok((old_id, new_id))
}

fn first_empty_index(paleta: &PaletaData) -> Result<u32, String> {
    let total = paleta.rows * paleta.cols;
    (1..=total)
        .find(|i| !paleta.botones.iter().any(|b| b.index == *i))
        .ok_or("no_empty_button".to_string())
}

fn move_button(btn: &mut ButtonData, paleta_id: &str, index: u32) {
    btn.index = index;
    btn.id = format!("{}_btn_{}", paleta_id, index);
}

#[cfg(test)]
mod tests {
    use super::{first_empty_index, move_button};
    use crate::button_defaults::new_button;
    use crate::types::PaletaData;

    #[test]
    fn first_empty_uses_lowest_available_slot() {
        let paleta = fixture(2, 3, &[1, 2, 4]);
        assert_eq!(first_empty_index(&paleta).unwrap(), 3);
    }

    #[test]
    fn full_paleta_rejects_move_target() {
        let paleta = fixture(1, 2, &[1, 2]);
        assert_eq!(first_empty_index(&paleta).unwrap_err(), "no_empty_button");
    }

    #[test]
    fn move_button_rewrites_index_and_id() {
        let mut button = new_button("a", 1, "A", "#111", "#fff");
        move_button(&mut button, "b", 4);
        assert_eq!(button.index, 4);
        assert_eq!(button.id, "b_btn_4");
    }

    fn fixture(rows: u32, cols: u32, indexes: &[u32]) -> PaletaData {
        PaletaData {
            id: "paleta".to_string(),
            nombre: "Test".to_string(),
            rows,
            cols,
            audio_out: String::new(),
            shortcut: String::new(),
            tab_bg: String::new(),
            tab_text: String::new(),
            botones: indexes
                .iter()
                .map(|i| new_button("paleta", *i, "B", "#111", "#fff"))
                .collect(),
        }
    }
}
