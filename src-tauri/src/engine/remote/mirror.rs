use crate::{
    core::AppState,
    engine::{cache::warm::warm_visible_tab, input::actions, persist::config_io},
    model::AppConfig,
};
use serde_json::{json, Value};
use tauri::Emitter;

pub(super) fn select_tab(
    app: &tauri::AppHandle,
    state: &AppState,
    profile: &str,
    palette: &str,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    if !can_select(&config, profile, palette) {
        return Err("remote_stale".into());
    }
    if config.active_paleta().is_some_and(|tab| tab.id == palette) {
        return Ok(());
    }
    let mut next = config.clone();
    actions::activate_paleta(&mut next, palette)?;
    config_io::save_config(&next).map_err(|_| "remote_command")?;
    *config = next;
    drop(config);
    warm_visible_tab(state);
    let _ = app.emit("global-shortcut-refresh", ());
    Ok(())
}

fn can_select(config: &AppConfig, profile: &str, palette: &str) -> bool {
    config.active_profile().is_some_and(|active| {
        active.id == profile
            && active
                .paletas
                .iter()
                .any(|tab| tab.id == palette && !tab.obs)
    })
}

pub(super) fn palette_snapshot(config: &AppConfig) -> Value {
    let profile = config.active_profile();
    let palette = config.active_paleta();
    let tabs = profile
        .map(|active| {
            active
                .paletas
                .iter()
                .map(|tab| {
                    json!({
                        "id": tab.id, "name": tab.nombre, "color_bg": tab.tab_bg,
                        "color_text": tab.tab_text, "enabled": !tab.obs,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let buttons = palette
        .map(|tab| {
            tab.botones.iter().map(|button| json!({
            "id": button.id, "index": button.index, "label": button.label, "name": button.name,
            "color_bg": button.color_bg, "color_text": button.color_text,
            "duration_str": button.duration_str, "visual": button.visual,
            "enabled": !tab.obs && button.type_field == "audio" && !button.path.is_empty(),
        })).collect::<Vec<_>>()
        })
        .unwrap_or_default();
    json!({
        "profile_id": profile.map(|p| &p.id), "profile": profile.map(|p| &p.name),
        "palette": palette.map(|p| &p.id), "name": palette.map(|p| &p.nombre),
        "columns": palette.map(|p| p.cols).unwrap_or(5),
        "rows": palette.map(|p| p.rows).unwrap_or(5),
        "tabs": tabs, "buttons": buttons, "language": config.language, "theme": config.theme,
        "unavailable": palette.is_some_and(|p| p.obs),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_lists_tabs_in_order_and_omits_local_paths() {
        let mut config = AppConfig::default();
        let profile = &mut config.profiles[0];
        let mut second = profile.paletas[0].clone();
        second.id = "second".into();
        second.nombre = "Segunda".into();
        second.botones.push(
            serde_json::from_value(json!({
                "id": "second_btn_1", "index": 1, "label": "Efecto", "path": "private.wav",
                "color_bg": "#123456", "color_text": "#ffffff"
            }))
            .unwrap(),
        );
        profile.paletas.push(second);
        profile.active_paleta_id = "second".into();
        let id = profile.id.clone();
        assert!(can_select(&config, &id, "second"));
        assert!(!can_select(&config, "other", "second"));
        let snapshot = palette_snapshot(&config);
        assert_eq!(snapshot["tabs"][1]["name"], "Segunda");
        assert_eq!(snapshot["buttons"][0]["enabled"], true);
        assert!(!snapshot.to_string().contains("private.wav"));
    }
}
