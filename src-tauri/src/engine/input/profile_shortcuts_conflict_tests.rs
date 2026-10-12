use super::{profile_shortcuts::*, profile_shortcuts_tests::configured, rules};
use crate::domain::button::defaults::new_button;

#[test]
fn rejects_reserved_invalid_and_duplicate_keys_before_mutation() {
    let cfg = configured();
    for (key, error) in [
        ("Ctrl+Z", "reserved_system_shortcut"),
        ("Control+Alt+Z", "reserved_system_shortcut"),
        ("Enter", "reserved_system_shortcut"),
        ("Escape", "reserved_system_shortcut"),
        ("NotAKey", "invalid_keyboard_shortcut"),
        ("Control+F10", "duplicate_keyboard_shortcut"),
    ] {
        let mut proposed = cfg.profiles[0].audio.clone();
        proposed.key_player_play = key.into();
        assert_eq!(validate(&cfg, &proposed).unwrap_err(), error);
        assert_eq!(cfg.profiles[0].audio.key_player_play, "Ctrl+F9");
    }
}

#[test]
fn swapping_assignments_validates_the_whole_proposed_set() {
    let cfg = configured();
    let mut proposed = cfg.profiles[0].audio.clone();
    std::mem::swap(&mut proposed.key_player_play, &mut proposed.key_player_stop);
    assert!(validate(&cfg, &proposed).is_ok());
    proposed.key_stop = "Ctrl+F9".into();
    assert_eq!(
        validate(&cfg, &proposed).unwrap_err(),
        "duplicate_keyboard_shortcut"
    );
}

#[test]
fn conflicts_include_tabs_grid_and_both_fixed_collections() {
    for target in 0..4 {
        let mut cfg = configured();
        let mut button = new_button("paleta_1", 1, "Test", "#000000", "#ffffff");
        button.shortcut = "Control+F9".into();
        match target {
            0 => cfg.profiles[0].paletas[0].shortcut = button.shortcut,
            1 => cfg.profiles[0].paletas[0].botones.push(button),
            2 => cfg.profiles[0].fixed_buttons.push(button),
            _ => cfg.fixed_panel.global_buttons.push(button),
        }
        assert_eq!(
            validate(&cfg, &cfg.profiles[0].audio).unwrap_err(),
            "keyboard_shortcut_in_use"
        );
    }
}

#[test]
fn player_keys_cannot_be_taken_by_buttons_or_tabs() {
    let mut cfg = configured();
    assert!(rules::apply_button_shortcut(&mut cfg, 1, "Ctrl+F9", true)
        .unwrap_err()
        .starts_with("shortcut_blocked_global|"));
    let profile = cfg.profiles[0].clone();
    assert!(
        rules::apply_tab_shortcut(&mut cfg, &profile.id, "paleta_1", "Ctrl+F10", true)
            .unwrap_err()
            .starts_with("shortcut_blocked_global|")
    );
    assert!(rules::block_profile_keys(&profile, "Control+F11").is_err());
    assert!(rules::block_profile_keys(&profile, "").is_ok());
}
