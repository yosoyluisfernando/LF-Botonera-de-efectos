use super::*;
use crate::domain::button::defaults::new_button;
use crate::domain::export::lfa_format::{from_lfa_paleta, to_lfa_paleta, LfaPaleta};
use crate::model::{AppConfig, ButtonData};
fn gradient() -> ButtonStyle {
    ButtonStyle {
        gradient: "diagonal".into(),
        bg2: "#0033FF".into(),
        text2: "#FFD700".into(),
        exact: true,
    }
}
#[test]
fn old_button_json_does_not_gain_style() {
    let b = new_button("p", 1, "Audio", "#123456", "#FFFFFF");
    let json = serde_json::to_value(&b).unwrap();
    assert!(json.get("style").is_none());
    let old: ButtonData = serde_json::from_value(json.clone()).unwrap();
    assert!(old.style.is_default());
    assert_eq!(serde_json::to_value(old).unwrap(), json);
}
#[test]
fn validates_every_supported_gradient_and_hex_case() {
    for direction in GRADIENTS {
        let mut s = gradient();
        s.gradient = direction.into();
        assert!(s.validate_colors("#aaFF01", "#ffffff").is_ok());
    }
    assert!(ButtonStyle::default()
        .validate_colors("#123456", "#FFFFFF")
        .is_ok());
}
#[test]
fn rejects_invalid_style_at_deserialization_boundary() {
    for json in [
        r##"{"gradient":"radial","bg2":"#000000"}"##,
        r##"{"gradient":"vertical"}"##,
        r##"{"bg2":"red"}"##,
        r##"{"text2":"#123"}"##,
        r##"{"text2":"url(x)"}"##,
    ] {
        assert!(serde_json::from_str::<ButtonStyle>(json).is_err(), "{json}");
    }
    assert!(gradient().validate_colors("red", "#FFFFFF").is_err());
    assert!(gradient().validate_colors("#000000", "#FFF").is_err());
}
#[test]
fn native_config_preserves_grid_fixed_and_player_style() {
    let mut cfg = AppConfig::default();
    let mut b = new_button("p", 1, "Audio", "#FF0000", "#FFFFFF");
    b.style = gradient();
    cfg.profiles[0].paletas[0].botones.push(b.clone());
    cfg.profiles[0].fixed_buttons.push(b.clone());
    cfg.fixed_panel.global_buttons.push(b.clone());
    cfg.player.tracks.push(b);
    let restored: AppConfig = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(restored.profiles[0].paletas[0].botones[0].style, gradient());
    assert_eq!(restored.profiles[0].fixed_buttons[0].style, gradient());
    assert_eq!(restored.fixed_panel.global_buttons[0].style, gradient());
    assert_eq!(restored.player.tracks[0].style, gradient());
}
#[test]
fn shared_tab_format_roundtrip_preserves_style_and_empty_cells() {
    let mut cfg = AppConfig::default();
    let tab = &mut cfg.profiles[0].paletas[0];
    let mut b = new_button(&tab.id, 1, "Audio", "#FF0000", "#FFFFFF");
    b.path = "aplausos.wav".into();
    b.style = gradient();
    tab.botones.push(b);
    let json = serde_json::to_value(to_lfa_paleta(tab)).unwrap();
    assert_eq!(json["botones"][0]["style"]["gradient"], "diagonal");
    assert!(json["botones"][1].get("style").is_none());
    let encoded: LfaPaleta = serde_json::from_value(json).unwrap();
    let restored = from_lfa_paleta(encoded, "dest".into());
    assert_eq!(restored.botones.len(), 1);
    assert_eq!(restored.botones[0].style, gradient());
    assert_eq!(restored.botones[0].color_bg, "#FF0000");
}
#[test]
fn custom_solid_and_text_only_gradient_survive_roundtrip() {
    for s in [
        ButtonStyle {
            exact: true,
            ..Default::default()
        },
        ButtonStyle {
            text2: "#123456".into(),
            exact: true,
            ..Default::default()
        },
    ] {
        let restored: ButtonStyle =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(restored, s);
    }
}
