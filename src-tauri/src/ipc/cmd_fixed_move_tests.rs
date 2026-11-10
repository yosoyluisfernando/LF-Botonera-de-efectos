use super::*;
use crate::domain::button::defaults::new_button;

fn grid(cfg: &AppConfig) -> &Vec<ButtonData> {
    &cfg.profiles[0].paletas[0].botones
}
fn fixed(cfg: &AppConfig) -> &Vec<ButtonData> {
    &cfg.fixed_panel.global_buttons
}
fn put_grid(cfg: &mut AppConfig, index: u32) {
    cfg.profiles[0].paletas[0]
        .botones
        .push(new_button("paleta_1", index, "G", "#111", "#fff"));
}
fn put_fixed(cfg: &mut AppConfig, index: u32) {
    cfg.fixed_panel
        .global_buttons
        .push(new_button("fixed_global", index, "F", "#222", "#fff"));
}

#[test]
fn grid_to_empty_fixed_moves_button() {
    let mut cfg = AppConfig::default();
    put_grid(&mut cfg, 3);
    let maps = relocate(&mut cfg, "paleta_1", 3, 1).unwrap();
    assert!(grid(&cfg).is_empty());
    assert_eq!(fixed(&cfg)[0].id, "fixed_global_btn_1");
    assert_eq!(
        maps,
        vec![("paleta_1_btn_3".into(), "fixed_global_btn_1".into())]
    );
}

#[test]
fn occupied_slots_swap_sides_and_ids() {
    let mut cfg = AppConfig::default();
    put_grid(&mut cfg, 3);
    put_fixed(&mut cfg, 1);
    let maps = relocate(&mut cfg, "paleta_1", 3, 1).unwrap();
    assert_eq!(grid(&cfg)[0].id, "paleta_1_btn_3");
    assert_eq!(grid(&cfg)[0].label, "F");
    assert_eq!(fixed(&cfg)[0].id, "fixed_global_btn_1");
    assert_eq!(fixed(&cfg)[0].label, "G");
    assert_eq!(maps.len(), 2);
}

#[test]
fn fixed_to_empty_grid_moves_button() {
    let mut cfg = AppConfig::default();
    put_fixed(&mut cfg, 1);
    let maps = relocate(&mut cfg, "paleta_1", 2, 1).unwrap();
    assert!(fixed(&cfg).is_empty());
    assert_eq!(grid(&cfg)[0].id, "paleta_1_btn_2");
    assert_eq!(
        maps,
        vec![("fixed_global_btn_1".into(), "paleta_1_btn_2".into())]
    );
}

#[test]
fn obs_palette_is_rejected_before_any_fixed_button_is_removed() {
    let mut cfg = AppConfig::default();
    cfg.profiles[0].paletas[0].obs = true;
    put_fixed(&mut cfg, 1);
    assert_eq!(
        relocate(&mut cfg, "paleta_1", 1, 1).unwrap_err(),
        "obs_separate_palettes"
    );
    assert_eq!(fixed(&cfg).len(), 1);
    assert!(grid(&cfg).is_empty());
}
