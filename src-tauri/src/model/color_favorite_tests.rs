use super::*;
use crate::model::AppConfig;
fn favorite(name: &str) -> ColorFavorite {
    ColorFavorite {
        name: name.into(),
        color_bg: "#598042".into(),
        color_text: "#ffffff".into(),
        style: ButtonStyle {
            gradient: "diagonal_inv".into(),
            bg2: "#021927".into(),
            text2: "#ffd700".into(),
            exact: true,
        },
    }
}
#[test]
fn old_configuration_has_no_favorites_and_keeps_its_json_shape() {
    let json = serde_json::to_value(AppConfig::default()).unwrap();
    assert!(json.get("color_favorites").is_none());
    let cfg: AppConfig = serde_json::from_value(json).unwrap();
    assert!(cfg.color_favorites.is_empty());
}
#[test]
fn favorites_persist_globally_without_changing_buttons() {
    let mut cfg = AppConfig::default();
    let profiles = serde_json::to_value(&cfg.profiles).unwrap();
    add(&mut cfg.color_favorites, favorite(" Verde y noche ")).unwrap();
    let restored: AppConfig = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(restored.color_favorites[0].name, "Verde y noche");
    assert_eq!(restored.color_favorites[0].style.text2, "#FFD700");
    assert_eq!(serde_json::to_value(restored.profiles).unwrap(), profiles);
}
#[test]
fn duplicate_names_do_not_overwrite_saved_colors() {
    let mut list = vec![];
    add(&mut list, favorite("Radio")).unwrap();
    let before = list.clone();
    let mut other = favorite(" radio ");
    other.color_bg = "#000000".into();
    assert_eq!(add(&mut list, other).unwrap_err(), "color_favorite_exists");
    assert_eq!(list, before);
}
#[test]
fn invalid_name_and_colors_are_rejected_before_mutation() {
    let mut list = vec![favorite("Uno")];
    let before = list.clone();
    for name in ["".to_string(), "  ".into(), "a".repeat(81), "a\nb".into()] {
        assert!(add(&mut list, favorite(&name)).is_err());
    }
    let mut bad = favorite("Malo");
    bad.color_bg = "red".into();
    assert!(add(&mut list, bad).is_err());
    assert_eq!(list, before);
}
#[test]
fn deletion_is_exact_and_keeps_other_favorites() {
    let mut list = vec![favorite("Uno"), favorite("Dos")];
    assert!(remove(&mut list, "missing").is_err());
    remove(&mut list, "Uno").unwrap();
    assert_eq!(list, vec![favorite("Dos")]);
}
