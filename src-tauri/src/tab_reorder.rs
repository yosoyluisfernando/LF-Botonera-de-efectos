/// Modulo: tab_reorder.rs
/// Proposito: reordenar pestanas dentro de un perfil sin cambiar sus ids ni datos.
use super::AppState;
use crate::config;
use crate::types::AppConfig;

#[tauri::command]
pub fn reorder_paletas(
    profile_id: String,
    from_paleta_id: String,
    to_paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<AppConfig, String> {
    if from_paleta_id == to_paleta_id {
        return Ok(state.config.lock().unwrap().clone());
    }

    let mut cfg = state.config.lock().unwrap();
    let profile_pos = cfg
        .profiles
        .iter()
        .position(|p| p.id == profile_id)
        .ok_or("profile_not_found")?;
    let (from, to) = paleta_positions(&cfg, profile_pos, &from_paleta_id, &to_paleta_id)?;

    state.history.lock().unwrap().remember(&cfg);
    reorder_in_profile(&mut cfg, profile_pos, from, to);
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

fn paleta_positions(
    cfg: &AppConfig,
    profile_pos: usize,
    from_paleta_id: &str,
    to_paleta_id: &str,
) -> Result<(usize, usize), String> {
    let paletas = &cfg.profiles[profile_pos].paletas;
    let from = paletas
        .iter()
        .position(|p| p.id == from_paleta_id)
        .ok_or("tab_not_found")?;
    let to = paletas
        .iter()
        .position(|p| p.id == to_paleta_id)
        .ok_or("tab_not_found")?;
    Ok((from, to))
}

fn reorder_in_profile(cfg: &mut AppConfig, profile_pos: usize, from: usize, to: usize) {
    let paletas = &mut cfg.profiles[profile_pos].paletas;
    let paleta = paletas.remove(from);
    paletas.insert(insertion_index(from, to), paleta);
}

fn insertion_index(_from: usize, to: usize) -> usize {
    to
}

#[cfg(test)]
mod tests {
    use super::insertion_index;

    #[test]
    fn moving_right_occupies_target_position() {
        assert_eq!(insertion_index(0, 2), 2);
    }

    #[test]
    fn moving_left_inserts_at_target_index() {
        assert_eq!(insertion_index(3, 1), 1);
    }
}
