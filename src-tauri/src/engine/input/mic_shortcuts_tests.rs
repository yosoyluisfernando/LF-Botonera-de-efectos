use super::{
    keyboard,
    profile_shortcuts::{self, ProfileAction},
};
use crate::domain::export::lfa_format::{from_lfa_profile, to_lfa_profile};
use crate::ipc::cmd_local_shortcuts::{resolve_local_action, LocalShortcutAction};
use crate::model::AppConfig;

#[test]
fn mic_uses_shared_resolution_and_global_scope() {
    let mut cfg = AppConfig::default();
    cfg.profiles[0].audio.key_mic = "Ctrl+F8".into();
    assert_eq!(
        resolve_local_action(&cfg, "Ctrl+F8").unwrap(),
        LocalShortcutAction::Profile(ProfileAction::Mic)
    );
    cfg.profiles[0].audio.global_keys = true;
    let shortcut = profile_shortcuts::parse("Ctrl+F8").unwrap();
    assert_eq!(
        keyboard::resolve_action(&cfg, &shortcut).unwrap(),
        keyboard::ShortcutAction::Profile(ProfileAction::Mic)
    );
    assert_eq!(
        resolve_local_action(&cfg, "Ctrl+F8").unwrap(),
        LocalShortcutAction::None
    );
    assert_eq!(keyboard::collect_keys(&cfg), vec!["Ctrl+F8"]);
}

#[test]
fn mic_conflicts_are_validated_with_other_actions_and_tabs() {
    let cfg = AppConfig::default();
    let mut audio = cfg.profiles[0].audio.clone();
    audio.key_mic = "Ctrl+F8".into();
    audio.key_player_play = "Control+F8".into();
    assert_eq!(
        profile_shortcuts::validate(&cfg, &audio),
        Err("duplicate_keyboard_shortcut".into())
    );
    audio.key_player_play.clear();
    assert!(profile_shortcuts::validate(&cfg, &audio).is_ok());
    let mut cfg = cfg;
    cfg.profiles[0].paletas[0].shortcut = "Ctrl+F8".into();
    assert_eq!(
        profile_shortcuts::validate(&cfg, &audio),
        Err("keyboard_shortcut_in_use".into())
    );
}

#[test]
fn profile_export_preserves_optional_mic_shortcut() {
    let mut cfg = AppConfig::default();
    cfg.profiles[0].audio.key_mic = "Ctrl+F8".into();
    let exported = to_lfa_profile(&cfg.profiles[0]);
    let restored = from_lfa_profile(exported, "restored".into());
    assert_eq!(restored.audio.key_mic, "Ctrl+F8");
}
