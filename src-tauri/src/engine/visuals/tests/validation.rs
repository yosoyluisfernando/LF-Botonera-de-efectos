use super::super::*;
use crate::model::ButtonVisual;

#[test]
fn button_visual_contract_is_strict() {
    let valid = ButtonVisual {
        kind: "emoji".into(),
        value: "👏".into(),
        mode: "visual_text".into(),
    };
    assert!(validate_button_visual(&valid).is_ok());
    let unknown = ButtonVisual {
        value: "no-es-un-emoji".into(),
        ..valid.clone()
    };
    assert_eq!(
        validate_button_visual(&unknown).unwrap_err(),
        "invalid_visual_value"
    );
    let hidden = ButtonVisual {
        mode: "text".into(),
        ..valid
    };
    assert!(validate_button_visual(&hidden).is_ok());
}

#[test]
fn bundled_monochrome_values_are_validated() {
    let basic = ButtonVisual {
        kind: "basic".into(),
        value: "tabler:animals:dog".into(),
        mode: "visual_text".into(),
    };
    assert!(validate_button_visual(&basic).is_ok());
    let unknown = ButtonVisual {
        value: "tabler:animals:not-bundled".into(),
        ..basic
    };
    assert_eq!(
        validate_button_visual(&unknown).unwrap_err(),
        "invalid_visual_value"
    );
}

#[test]
fn original_icon_can_be_presented_as_text_only() {
    let visual = ButtonVisual {
        mode: "text".into(),
        ..ButtonVisual::default()
    };
    assert!(validate_button_visual(&visual).is_ok());
}

#[test]
fn legacy_material_values_remain_valid() {
    let visual = ButtonVisual {
        kind: "basic".into(),
        value: "radio".into(),
        mode: "visual_text".into(),
    };
    assert!(validate_button_visual(&visual).is_ok());
}
