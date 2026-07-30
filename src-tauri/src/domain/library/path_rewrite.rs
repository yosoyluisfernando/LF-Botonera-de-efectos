//! Actualiza referencias internas cuando un audio cambia de nombre físico.
use crate::engine::persist::db;
use crate::model::{AppConfig, ButtonData};

pub fn replace_in_config(config: &mut AppConfig, old_path: &str, new_path: &str) -> usize {
    let old_key = db::normalize_key(old_path);
    let mut changed = 0;
    changed += replace_buttons(&mut config.fixed_panel.global_buttons, &old_key, new_path);
    changed += replace_buttons(&mut config.player.tracks, &old_key, new_path);
    for profile in &mut config.profiles {
        changed += replace_buttons(&mut profile.fixed_buttons, &old_key, new_path);
        for palette in &mut profile.paletas {
            changed += replace_buttons(&mut palette.botones, &old_key, new_path);
        }
    }
    changed
}

fn replace_buttons(buttons: &mut [ButtonData], old_key: &str, new_path: &str) -> usize {
    let mut changed = 0;
    for button in buttons {
        if button.type_field == "audio" && db::normalize_key(&button.path) == old_key {
            button.path = new_path.to_string();
            changed += 1;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::button::defaults::new_button;

    fn audio(path: &str, id: &str) -> ButtonData {
        let mut button = new_button(id, 0, "", "", "");
        button.path = path.into();
        button
    }

    #[test]
    fn rewrites_every_config_surface_without_touching_labels() {
        let old = "C:/audio/Risa.mp3";
        let new = "C:/audio/Risa larga.mp3";
        let mut config = AppConfig::default();
        config.profiles[0].paletas[0]
            .botones
            .push(audio(old, "grid"));
        config.profiles[0].fixed_buttons.push(audio(old, "fixed"));
        config.fixed_panel.global_buttons.push(audio(old, "global"));
        config.player.tracks.push(audio(old, "player"));
        config.player.tracks[0].label = "Nombre personalizado".into();

        assert_eq!(replace_in_config(&mut config, old, new), 4);
        assert_eq!(config.player.tracks[0].path, new);
        assert_eq!(config.player.tracks[0].label, "Nombre personalizado");
    }

    #[test]
    fn ignores_non_audio_and_unrelated_paths() {
        let mut config = AppConfig::default();
        let mut special = audio("C:/audio/Risa.mp3", "clock");
        special.type_field = "time".into();
        config.player.tracks.push(special);
        config
            .fixed_panel
            .global_buttons
            .push(audio("C:/audio/Otro.mp3", "other"));

        assert_eq!(
            replace_in_config(&mut config, "C:/audio/Risa.mp3", "C:/audio/Nueva.mp3"),
            0
        );
    }
}
