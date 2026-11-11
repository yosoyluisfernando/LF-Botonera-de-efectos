use super::button_snapshot;
use crate::{domain::button::defaults::new_button, model::AppConfig};

#[test]
fn an_imported_button_resolves_in_the_active_profile() {
    let mut cfg = AppConfig::default();
    let mut first = new_button("paleta_imp_2", 1, "Original", "#000000", "#ffffff");
    first.path = "original.wav".into();
    cfg.profiles[0].paletas[0].id = "paleta_imp_2".into();
    cfg.profiles[0].paletas[0].botones.push(first.clone());
    let mut imported = cfg.profiles[0].clone();
    imported.id = "perfil_imp_2".into();
    imported.paletas[0].botones[0].path = "importado.wav".into();
    cfg.active_profile_id = imported.id.clone();
    cfg.profiles.push(imported);

    let (button, _) = button_snapshot(&cfg, &first.id).unwrap();
    assert_eq!(button.path, "importado.wav");
}

#[test]
fn a_button_from_another_profile_is_not_triggered() {
    let mut cfg = AppConfig::default();
    cfg.profiles[0].paletas[0].botones.push(
        new_button("original", 1, "Original", "#000000", "#ffffff"));
    let mut imported = cfg.profiles[0].clone();
    imported.id = "perfil_imp_2".into();
    imported.paletas[0].botones.clear();
    cfg.active_profile_id = imported.id.clone();
    cfg.profiles.push(imported);
    assert!(button_snapshot(&cfg, "original_btn_1").is_err());
}
