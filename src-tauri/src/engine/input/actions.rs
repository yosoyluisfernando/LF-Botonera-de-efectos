use crate::model::AppConfig;

pub fn cycle_paleta(cfg: &mut AppConfig, offset: i32) -> Result<(), String> {
    let profile = cfg
        .active_profile_mut()
        .ok_or("Perfil activo no encontrado")?;
    let len = profile.paletas.len() as i32;
    if len == 0 {
        return Err("El perfil no tiene pestanas".to_string());
    }
    let current = profile
        .paletas
        .iter()
        .position(|p| p.id == profile.active_paleta_id)
        .unwrap_or(0) as i32;
    let next = (current + offset).rem_euclid(len) as usize;
    profile.active_paleta_id = profile.paletas[next].id.clone();
    Ok(())
}

pub fn activate_paleta(cfg: &mut AppConfig, paleta_id: &str) -> Result<bool, String> {
    let profile = cfg
        .active_profile_mut()
        .ok_or("Perfil activo no encontrado")?;
    if profile.paletas.iter().any(|p| p.id == paleta_id) {
        profile.active_paleta_id = paleta_id.to_string();
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn play_by_shortcut(cfg: &AppConfig, key: &str) -> Result<Option<String>, String> {
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    let fixed = if cfg.fixed_panel.scope == "profile" {
        &profile.fixed_buttons
    } else {
        &cfg.fixed_panel.global_buttons
    };
    if let Some(btn) = fixed.iter().find(|btn| same_key(&btn.shortcut, key)) {
        return Ok(Some(btn.id.clone()));
    }
    let Some(active) = profile
        .paletas
        .iter()
        .find(|p| p.id == profile.active_paleta_id)
    else {
        return Ok(None);
    };
    Ok(active
        .botones
        .iter()
        .find(|btn| same_key(&btn.shortcut, key))
        .map(|btn| btn.id.clone()))
}

pub fn same_key(saved: &str, key: &str) -> bool {
    !saved.trim().is_empty() && saved.trim().eq_ignore_ascii_case(key.trim())
}
