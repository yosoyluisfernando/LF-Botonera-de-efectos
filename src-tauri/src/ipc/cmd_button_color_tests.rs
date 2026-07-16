//! Pruebas de pintar varios botones a la vez.
use super::*;
use crate::domain::button::defaults::new_button;

fn cfg_with(colors: &[&str]) -> AppConfig {
    let mut cfg = AppConfig::default();
    let paleta = &mut cfg.profiles[0].paletas[0];
    for (i, c) in colors.iter().enumerate() {
        let mut b = new_button("paleta_1", i as u32 + 1, "X", c, "#FFFFFF");
        b.path = format!("C:/{i}.mp3");
        paleta.botones.push(b);
    }
    cfg
}

fn colors_of(cfg: &AppConfig) -> Vec<String> {
    cfg.profiles[0].paletas[0].botones.iter().map(|b| b.color_bg.clone()).collect()
}

/// Lo pedido: seleccionas varios y quedan del mismo color.
#[test]
fn paints_only_the_selected_buttons() {
    let mut cfg = cfg_with(&["#111111", "#222222", "#333333"]);

    let n = paint(&mut cfg, &[1, 3], "#DB2424", "#FFFFFF", "grid").unwrap();

    assert_eq!(n, 2);
    assert_eq!(colors_of(&cfg), ["#DB2424", "#222222", "#DB2424"], "el 2 no se toca");
}

/// Un indice que ya no existe no puede tirar la operacion entera: entre
/// seleccionar y pintar la rejilla pudo cambiar.
#[test]
fn a_stale_index_does_not_lose_the_rest() {
    let mut cfg = cfg_with(&["#111111", "#222222"]);

    let n = paint(&mut cfg, &[1, 99], "#DB2424", "#FFFFFF", "grid").unwrap();

    assert_eq!(n, 1, "pinta el que existe");
    assert_eq!(colors_of(&cfg)[0], "#DB2424");
}

/// El grupo debe ser uno de los conocidos: la rejilla y el panel numeran aparte.
#[test]
fn an_unknown_group_is_rejected() {
    let mut cfg = cfg_with(&["#111111"]);
    assert!(paint(&mut cfg, &[1], "#DB2424", "#FFFFFF", "player").is_err());
}

/// El panel fijo tambien se puede pintar: es el mismo gesto y el mismo menu.
#[test]
fn it_also_paints_the_fixed_panel() {
    let mut cfg = AppConfig::default();
    let mut b = new_button("fixed_global", 1, "X", "#111111", "#FFFFFF");
    b.path = "C:/a.mp3".into();
    cfg.fixed_panel.global_buttons.push(b);

    let n = paint(&mut cfg, &[1], "#DB2424", "#FFFFFF", "fixed").unwrap();

    assert_eq!(n, 1);
    assert_eq!(cfg.fixed_panel.global_buttons[0].color_bg, "#DB2424");
}
