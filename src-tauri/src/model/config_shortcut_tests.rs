use super::AppConfig;

#[test]
fn remote_client_is_opt_in_and_preserves_explicit_preferences() {
    assert!(!AppConfig::default().remote_client_enabled);
    let old: AppConfig = serde_json::from_str("{}").unwrap();
    assert!(!old.remote_client_enabled);
    let enabled: AppConfig =
        serde_json::from_str(r#"{"remote_client_enabled":true}"#).unwrap();
    assert!(enabled.remote_client_enabled);
}

#[test]
fn older_configs_show_both_shortcut_labels_by_default() {
    let cfg: AppConfig = serde_json::from_str("{}").unwrap();
    assert!(cfg.show_keyboard_shortcut_labels);
    assert!(cfg.show_midi_shortcut_labels);
    assert!(AppConfig::default().show_keyboard_shortcut_labels);
    assert!(AppConfig::default().show_midi_shortcut_labels);
}

#[test]
fn label_preferences_round_trip_without_modifying_bindings() {
    let mut cfg = AppConfig::default();
    cfg.show_keyboard_shortcut_labels = false;
    cfg.show_midi_shortcut_labels = true;
    cfg.profiles[0].audio.key_stop = "Ctrl+F1".into();
    let restored: AppConfig = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
    assert!(!restored.show_keyboard_shortcut_labels);
    assert!(restored.show_midi_shortcut_labels);
    assert_eq!(restored.profiles[0].audio.key_stop, "Ctrl+F1");
}

#[test]
fn bottom_bar_defaults_visible_and_round_trips_when_hidden() {
    let old: AppConfig = serde_json::from_str("{}").unwrap();
    assert!(old.show_bottom_bar && AppConfig::default().show_bottom_bar);
    let mut cfg = old;
    cfg.show_bottom_bar = false;
    let restored: AppConfig = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
    assert!(!restored.show_bottom_bar);
}
