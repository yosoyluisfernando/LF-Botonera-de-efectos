//! Rutas cuyos metadatos técnicos siguen en uso fuera de la Biblioteca.
use crate::engine::persist::db;
use crate::model::{AppConfig, ButtonData};
use std::collections::HashSet;

pub fn from_config(config: &AppConfig) -> HashSet<String> {
    let mut paths = HashSet::new();
    collect_buttons(&mut paths, &config.fixed_panel.global_buttons);
    collect_buttons(&mut paths, &config.player.tracks);
    for profile in &config.profiles {
        collect_buttons(&mut paths, &profile.fixed_buttons);
        for palette in &profile.paletas {
            collect_buttons(&mut paths, &palette.botones);
        }
    }
    paths
}

fn collect_buttons(target: &mut HashSet<String>, buttons: &[ButtonData]) {
    target.extend(
        buttons
            .iter()
            .filter(|button| button.type_field == "audio" && !button.path.trim().is_empty())
            .map(|button| db::normalize_key(&button.path)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::button::defaults::new_button;

    #[test]
    fn protects_all_grid_fixed_panel_and_player_audio_paths() {
        let mut config = AppConfig::default();
        let mut grid = new_button("p", 0, "", "", "");
        grid.path = "C:/audio/grid.wav".into();
        config.profiles[0].paletas[0].botones.push(grid);
        let mut profile_fixed = new_button("p", 1, "", "", "");
        profile_fixed.path = "C:/audio/profile.wav".into();
        config.profiles[0].fixed_buttons.push(profile_fixed);
        let mut global_fixed = new_button("g", 0, "", "", "");
        global_fixed.path = "C:/audio/global.wav".into();
        config.fixed_panel.global_buttons.push(global_fixed);
        let mut player = new_button("q", 0, "", "", "");
        player.path = "C:/audio/player.wav".into();
        config.player.tracks.push(player);

        let paths = from_config(&config);

        for path in ["grid.wav", "profile.wav", "global.wav", "player.wav"] {
            assert!(paths.iter().any(|value| value.ends_with(path)));
        }
    }

    #[test]
    fn ignores_non_audio_and_empty_buttons() {
        let mut config = AppConfig::default();
        let mut special = new_button("p", 0, "", "", "");
        special.type_field = "time".into();
        special.path = "C:/audio/not-used.wav".into();
        config.profiles[0].paletas[0].botones.push(special);

        assert!(from_config(&config).is_empty());
    }
}
