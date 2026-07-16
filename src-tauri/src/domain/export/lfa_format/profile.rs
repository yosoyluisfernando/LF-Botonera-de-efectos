use super::paleta::{from_lfa_paleta, to_lfa_paleta};
use super::types::{LfaConfig, LfaKeys, LfaProfile};
use crate::model::{AudioConfig, PaletaData, ProfileData};

/// Perfil completo -> formato LFA, incluyendo config de salidas y atajos.
pub fn to_lfa_profile(p: &ProfileData) -> LfaProfile {
    LfaProfile {
        id: p.id.clone(),
        name: p.name.clone(),
        bg: p.bg.clone(),
        text: p.text.clone(),
        config: LfaConfig {
            out_main: p.audio.out_main.clone(),
            out_pre: p.audio.out_pre.clone(),
            keys: LfaKeys {
                stop_all: p.audio.key_stop.clone(),
                next: p.audio.key_next.clone(),
                prev: p.audio.key_prev.clone(),
            },
        },
        paletas: p.paletas.iter().map(to_lfa_paleta).collect(),
        fixed_buttons: p.fixed_buttons.clone(),
    }
}

/// Formato LFA -> perfil interno. `new_id` lo decide el caller.
pub fn from_lfa_profile(lfa: LfaProfile, new_id: String) -> ProfileData {
    let fixed_buttons = lfa
        .fixed_buttons
        .into_iter()
        .enumerate()
        .map(|(i, mut button)| {
            button.index = i as u32 + 1;
            button.id = format!("fixed_{}_btn_{}", new_id, button.index);
            button
        })
        .collect();
    let paletas: Vec<PaletaData> = lfa
        .paletas
        .into_iter()
        .enumerate()
        .map(|(i, p)| from_lfa_paleta(p, format!("{}_paleta_{}", new_id, i)))
        .collect();
    let first_pid = paletas.first().map(|p| p.id.clone()).unwrap_or_default();
    ProfileData {
        id: new_id,
        name: lfa.name,
        bg: lfa.bg,
        text: lfa.text,
        audio: AudioConfig {
            out_main: if lfa.config.out_main.is_empty() {
                "default".to_string()
            } else {
                lfa.config.out_main
            },
            out_pre: lfa.config.out_pre,
            global_keys: false,
            key_stop: lfa.config.keys.stop_all,
            key_next: lfa.config.keys.next,
            key_prev: lfa.config.keys.prev,
            playback_mode: "normal".to_string(),
            solo_mode: false,
            master_volume: 1.0,
            master_volume_remember: false,
            master_volume_boost: false,
        },
        active_paleta_id: first_pid,
        paletas,
        fixed_buttons,
    }
}
