use super::midi_binding::{conflict, name, same};
use crate::model::{AppConfig, MidiBinding, ProfileData};

pub fn apply_button(
    cfg: &mut AppConfig,
    index: u32,
    binding: &MidiBinding,
    replace: bool,
) -> Result<(), String> {
    if binding.is_empty() {
        return Ok(());
    }
    block_global(cfg, binding)?;
    clear_tab(cfg, binding, replace)?;
    clear_fixed(cfg, None, binding, replace)?;
    let profile = active_profile_mut(cfg)?;
    let active_id = profile.active_paleta_id.clone();
    let paleta = profile
        .paletas
        .iter_mut()
        .find(|item| item.id == active_id)
        .ok_or("Pestana activa no encontrada")?;
    for button in paleta
        .botones
        .iter_mut()
        .filter(|button| button.index != index)
    {
        if same(&button.midi, binding) {
            let target = format!("{} {}", button.index, name(&button.name, &button.label));
            if !replace {
                return Err(conflict("midi_conflict_button", binding, &target));
            }
            button.midi = MidiBinding::default();
        }
    }
    Ok(())
}

pub fn apply_fixed(
    cfg: &mut AppConfig,
    index: u32,
    binding: &MidiBinding,
    replace: bool,
) -> Result<(), String> {
    if binding.is_empty() {
        return Ok(());
    }
    block_global(cfg, binding)?;
    clear_tab(cfg, binding, replace)?;
    clear_buttons_any(cfg, binding, replace)?;
    clear_fixed(cfg, Some(index), binding, replace)
}

pub fn apply_tab(
    cfg: &mut AppConfig,
    profile_id: &str,
    paleta_id: &str,
    binding: &MidiBinding,
    replace: bool,
) -> Result<(), String> {
    if binding.is_empty() {
        return Ok(());
    }
    block_global(cfg, binding)?;
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|item| item.id == profile_id)
        .ok_or("Perfil no encontrado")?;
    for tab in profile.paletas.iter_mut().filter(|tab| tab.id != paleta_id) {
        if same(&tab.midi, binding) {
            if !replace {
                return Err(conflict("midi_conflict_tab", binding, &tab.nombre));
            }
            tab.midi = MidiBinding::default();
        }
    }
    clear_buttons_in_profile(profile, binding, replace)?;
    clear_fixed(cfg, None, binding, replace)
}

pub fn validate_global(cfg: &AppConfig, key: &str, binding: &MidiBinding) -> Result<(), String> {
    if binding.is_empty() {
        return Ok(());
    }
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    let audio = &profile.audio;
    let blocked = match key {
        "stop" => same(&audio.midi_next, binding) || same(&audio.midi_prev, binding),
        "next" => same(&audio.midi_stop, binding) || same(&audio.midi_prev, binding),
        "prev" => same(&audio.midi_stop, binding) || same(&audio.midi_next, binding),
        _ => false,
    };
    if blocked {
        return Err(conflict("midi_blocked_global", binding, ""));
    }
    Ok(())
}

fn block_global(cfg: &AppConfig, binding: &MidiBinding) -> Result<(), String> {
    let profile = cfg.active_profile().ok_or("Perfil activo no encontrado")?;
    let audio = &profile.audio;
    if same(&audio.midi_stop, binding)
        || same(&audio.midi_next, binding)
        || same(&audio.midi_prev, binding)
    {
        return Err(conflict("midi_blocked_global", binding, ""));
    }
    Ok(())
}

fn clear_tab(cfg: &mut AppConfig, binding: &MidiBinding, replace: bool) -> Result<(), String> {
    let profile = active_profile_mut(cfg)?;
    for tab in &mut profile.paletas {
        if same(&tab.midi, binding) {
            if !replace {
                return Err(conflict("midi_conflict_tab", binding, &tab.nombre));
            }
            tab.midi = MidiBinding::default();
        }
    }
    Ok(())
}

fn clear_buttons_any(
    cfg: &mut AppConfig,
    binding: &MidiBinding,
    replace: bool,
) -> Result<(), String> {
    let profile = active_profile_mut(cfg)?;
    clear_buttons_in_profile(profile, binding, replace)
}

fn clear_buttons_in_profile(
    profile: &mut ProfileData,
    binding: &MidiBinding,
    replace: bool,
) -> Result<(), String> {
    for paleta in &mut profile.paletas {
        for button in paleta
            .botones
            .iter_mut()
            .filter(|button| same(&button.midi, binding))
        {
            let target = format!("{} - {}", paleta.nombre, name(&button.name, &button.label));
            if !replace {
                return Err(conflict("midi_conflict_button_any", binding, &target));
            }
            button.midi = MidiBinding::default();
        }
    }
    Ok(())
}

fn clear_fixed(
    cfg: &mut AppConfig,
    except_index: Option<u32>,
    binding: &MidiBinding,
    replace: bool,
) -> Result<(), String> {
    let buttons = fixed_buttons_mut(cfg)?;
    for button in buttons
        .iter_mut()
        .filter(|button| except_index != Some(button.index) && same(&button.midi, binding))
    {
        let target = name(&button.name, &button.label);
        if !replace {
            return Err(conflict("midi_conflict_fixed", binding, &target));
        }
        button.midi = MidiBinding::default();
    }
    Ok(())
}

fn fixed_buttons_mut(cfg: &mut AppConfig) -> Result<&mut Vec<crate::model::ButtonData>, String> {
    if cfg.fixed_panel.scope == "profile" {
        Ok(&mut active_profile_mut(cfg)?.fixed_buttons)
    } else {
        Ok(&mut cfg.fixed_panel.global_buttons)
    }
}

fn active_profile_mut(cfg: &mut AppConfig) -> Result<&mut ProfileData, String> {
    cfg.active_profile_mut()
        .ok_or("Perfil activo no encontrado".into())
}
