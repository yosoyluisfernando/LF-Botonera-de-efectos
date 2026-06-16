/// Modulo: shortcut_rules.rs
/// Proposito: reglas centrales para asignar atajos sin conflictos peligrosos.
use crate::types::{AppConfig, ProfileData};

/// Valida y aplica la asignacion de atajo a un boton de la paleta activa.
pub fn apply_button_shortcut(
    cfg: &mut AppConfig,
    index: u32,
    key: &str,
    replace: bool,
) -> Result<(), String> {
    if key.trim().is_empty() {
        return Ok(());
    }
    block_system_key(key)?;
    let profile = active_profile_mut(cfg)?;
    block_profile_keys(profile, key)?;
    clear_tab_conflict(profile, key, replace)?;

    let active_id = profile.active_paleta_id.clone();
    let paleta = profile
        .paletas
        .iter_mut()
        .find(|p| p.id == active_id)
        .ok_or("Pestana activa no encontrada")?;

    if let Some(btn) = paleta
        .botones
        .iter_mut()
        .find(|b| b.index != index && same_key(&b.shortcut, key))
    {
        let target = format!("{} {}", btn.index, display_name(&btn.name, &btn.label));
        if !replace {
            return Err(conflict("shortcut_conflict_button", key, &target));
        }
        btn.shortcut.clear();
    }
    Ok(())
}

/// Valida y aplica la asignacion de atajo a una pestana del perfil indicado.
pub fn apply_tab_shortcut(
    cfg: &mut AppConfig,
    profile_id: &str,
    paleta_id: &str,
    key: &str,
    replace: bool,
) -> Result<(), String> {
    if key.trim().is_empty() {
        return Ok(());
    }
    block_system_key(key)?;
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    block_profile_keys(profile, key)?;

    if let Some(tab) = profile
        .paletas
        .iter_mut()
        .find(|p| p.id != paleta_id && same_key(&p.shortcut, key))
    {
        let target = tab.nombre.clone();
        if !replace {
            return Err(conflict("shortcut_conflict_tab", key, &target));
        }
        tab.shortcut.clear();
    }

    let mut first_button = None;
    for paleta in profile.paletas.iter_mut() {
        for btn in paleta
            .botones
            .iter_mut()
            .filter(|b| same_key(&b.shortcut, key))
        {
            if first_button.is_none() {
                first_button = Some(format!(
                    "{} - {}",
                    paleta.nombre,
                    display_name(&btn.name, &btn.label)
                ));
            }
            if replace {
                btn.shortcut.clear();
            }
        }
    }
    if let Some(target) = first_button {
        if !replace {
            return Err(conflict("shortcut_conflict_button_any", key, &target));
        }
    }
    Ok(())
}

fn active_profile_mut(cfg: &mut AppConfig) -> Result<&mut ProfileData, String> {
    let pid = cfg.active_profile_id.clone();
    cfg.profiles
        .iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado".to_string())
}

fn block_profile_keys(profile: &ProfileData, key: &str) -> Result<(), String> {
    let audio = &profile.audio;
    if same_key(&audio.key_stop, key)
        || same_key(&audio.key_next, key)
        || same_key(&audio.key_prev, key)
    {
        return Err(conflict("shortcut_blocked_global", key, ""));
    }
    Ok(())
}

pub fn is_reserved_system_key(key: &str) -> bool {
    ["Ctrl+Z", "Ctrl+Alt+Z"]
        .iter()
        .any(|item| same_key(item, key))
}

fn block_system_key(key: &str) -> Result<(), String> {
    if is_reserved_system_key(key) {
        return Err(conflict("shortcut_reserved_system", key, ""));
    }
    Ok(())
}

fn clear_tab_conflict(profile: &mut ProfileData, key: &str, replace: bool) -> Result<(), String> {
    if let Some(tab) = profile
        .paletas
        .iter_mut()
        .find(|p| same_key(&p.shortcut, key))
    {
        let target = tab.nombre.clone();
        if !replace {
            return Err(conflict("shortcut_conflict_tab", key, &target));
        }
        tab.shortcut.clear();
    }
    Ok(())
}

fn same_key(a: &str, b: &str) -> bool {
    !a.trim().is_empty() && a.trim().eq_ignore_ascii_case(b.trim())
}

fn display_name(name: &str, label: &str) -> String {
    if !name.trim().is_empty() {
        name.to_string()
    } else {
        label.to_string()
    }
}

fn conflict(code: &str, key: &str, target: &str) -> String {
    format!("{code}|{}|{}", key.trim(), target.replace('|', " "))
}
