use super::{keyboard, profile_shortcuts::*};
use crate::domain::export::lfa_format::{from_lfa_profile, to_lfa_profile, LfaProfile};
use crate::ipc::cmd_local_shortcuts::{resolve_local_action, LocalShortcutAction};
use crate::model::{AppConfig, AudioConfig};

pub(super) fn configured() -> AppConfig {
    let mut cfg = AppConfig::default();
    let audio = &mut cfg.profiles[0].audio;
    audio.key_player_play = "Ctrl+F9".into();
    audio.key_player_stop = "Ctrl+F10".into();
    audio.key_player_next = "Ctrl+F11".into();
    cfg
}

#[test]
fn old_configuration_has_no_player_shortcuts() {
    let audio: AudioConfig = serde_json::from_str(r#"{"key_stop":"F1"}"#).unwrap();
    assert!(audio.key_player_play.is_empty());
    assert!(audio.key_player_stop.is_empty());
    assert!(audio.key_player_next.is_empty());
    assert!(AudioConfig::default().key_player_play.is_empty());
    assert_eq!(audio.key_stop, "F1");
    let json = serde_json::to_value(&audio).unwrap();
    assert!(json.get("key_player_play").is_none());
}

#[test]
fn player_actions_resolve_locally_without_registering_global_keys() {
    let cfg = configured();
    for (key, action) in [
        ("Ctrl+F9", ProfileAction::PlayerPlay),
        ("Ctrl+F10", ProfileAction::PlayerStop),
        ("Ctrl+F11", ProfileAction::PlayerNext),
    ] {
        assert_eq!(
            resolve_local_action(&cfg, key).unwrap(),
            LocalShortcutAction::Profile(action)
        );
    }
    assert!(keyboard::collect_keys(&cfg).is_empty());
    assert!(keyboard::resolve_action(&cfg, &parse("Ctrl+F9").unwrap()).is_err());
}

#[test]
fn global_player_actions_are_not_also_handled_locally() {
    let mut cfg = configured();
    cfg.profiles[0].audio.global_keys = true;
    for (key, action) in [
        ("Ctrl+F9", ProfileAction::PlayerPlay),
        ("Ctrl+F10", ProfileAction::PlayerStop),
        ("Ctrl+F11", ProfileAction::PlayerNext),
    ] {
        assert!(keyboard::collect_keys(&cfg).contains(&key.to_string()));
        assert_eq!(
            keyboard::resolve_action(&cfg, &parse(key).unwrap()).unwrap(),
            keyboard::ShortcutAction::Profile(action)
        );
        assert_eq!(
            resolve_local_action(&cfg, key).unwrap(),
            LocalShortcutAction::None
        );
    }
}

#[test]
fn existing_actions_keep_their_meaning() {
    let mut cfg = configured();
    let audio = &mut cfg.profiles[0].audio;
    audio.key_stop = "F1".into();
    audio.key_next = "F2".into();
    audio.key_prev = "F3".into();
    for (key, action) in [
        ("F1", ProfileAction::StopAll),
        ("F2", ProfileAction::Cycle(1)),
        ("F3", ProfileAction::Cycle(-1)),
    ] {
        assert_eq!(
            resolve_local_action(&cfg, key).unwrap(),
            LocalShortcutAction::Profile(action)
        );
    }
}

#[test]
fn clearing_and_switching_profile_release_player_keys() {
    let mut cfg = configured();
    let mut other = cfg.profiles[0].clone();
    other.id = "other".into();
    other.audio = AudioConfig::default();
    cfg.profiles.push(other);
    cfg.active_profile_id = "other".into();
    assert_eq!(
        resolve_local_action(&cfg, "Ctrl+F9").unwrap(),
        LocalShortcutAction::None
    );
    cfg.active_profile_id = cfg.profiles[0].id.clone();
    cfg.profiles[0].audio.key_player_play.clear();
    cfg.profiles[0].audio.global_keys = true;
    assert!(!keyboard::collect_keys(&cfg).contains(&"Ctrl+F9".to_string()));
}

#[test]
fn player_shortcuts_survive_config_and_profile_export_round_trips() {
    let cfg = configured();
    let json = serde_json::to_string(&cfg).unwrap();
    let restored: AppConfig = serde_json::from_str(&json).unwrap();
    let exported = to_lfa_profile(&restored.profiles[0]);
    let json = serde_json::to_string(&exported).unwrap();
    let imported = from_lfa_profile(serde_json::from_str(&json).unwrap(), "imported".into());
    assert_eq!(imported.audio.key_player_play, "Ctrl+F9");
    assert_eq!(imported.audio.key_player_stop, "Ctrl+F10");
    assert_eq!(imported.audio.key_player_next, "Ctrl+F11");
}

#[test]
fn legacy_profile_import_leaves_player_shortcuts_empty() {
    let old: LfaProfile = serde_json::from_str(
        r#"{"name":"Old","paletas":[],
        "config":{"keys":{"stopAll":"F1"}}}"#,
    )
    .unwrap();
    let profile = from_lfa_profile(old, "legacy".into());
    assert_eq!(profile.audio.key_stop, "F1");
    assert!(profile.audio.key_player_play.is_empty());
    assert!(profile.audio.key_player_stop.is_empty());
    assert!(profile.audio.key_player_next.is_empty());
}
