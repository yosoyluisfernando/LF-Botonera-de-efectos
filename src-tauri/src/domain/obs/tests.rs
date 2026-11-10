use super::*;
use crate::model::obs::{ObsAction, ObsSnapshot};
#[test]
fn old_config_defaults_to_audio_and_obs_disabled() {
    let cfg = AppConfig::default();
    let mut value = serde_json::to_value(&cfg).unwrap();
    value.as_object_mut().unwrap().remove("obs");
    value["profiles"][0]["paletas"][0]
        .as_object_mut()
        .unwrap()
        .remove("obs");
    let restored: AppConfig = serde_json::from_value(value).unwrap();
    assert!(!restored.obs.enabled);
    assert!(!restored.profiles[0].paletas[0].obs);
}
#[test]
fn preview_and_transition_require_studio() {
    let s = ObsSnapshot::default();
    for kind in ["scene_preview", "transition"] {
        let a = ObsAction {
            kind: kind.into(),
            target: "Camera".into(),
            duration_ms: None,
            ..Default::default()
        };
        assert_eq!(actions::request(&a, &s).unwrap_err(), "obs_studio_required");
    }
}
#[test]
fn explicit_program_and_toggle_use_real_state() {
    let mut s = ObsSnapshot {
        recording: true,
        available: vec!["SetCurrentProgramScene".into(), "StopRecord".into()],
        ..Default::default()
    };
    let a = ObsAction {
        kind: "scene_program".into(),
        target: "Camera".into(),
        duration_ms: None,
        ..Default::default()
    };
    let (name, data) = actions::request(&a, &s).unwrap();
    assert_eq!(name, "SetCurrentProgramScene");
    assert_eq!(data["sceneName"], "Camera");
    let a = ObsAction {
        kind: "record_toggle".into(),
        ..Default::default()
    };
    assert_eq!(actions::request(&a, &s).unwrap().0, "StopRecord");
    s.recording = false;
    assert_eq!(actions::request(&a, &s).unwrap_err(), "obs_unsupported");
}
#[test]
fn obs_palettes_cannot_receive_audio_or_mix() {
    let cfg = AppConfig::default();
    let audio = &cfg.profiles[0].paletas[0];
    let mut obs = audio.clone();
    obs.obs = true;
    assert!(audio_paleta(&obs).is_err());
    assert!(compatible(audio, &obs).is_err());
    assert!(compatible(&obs, &obs).is_ok());
}
#[test]
fn shared_format_preserves_obs_without_credentials() {
    use crate::domain::{
        button::defaults::new_button,
        export::lfa_format::{from_lfa_paleta, to_lfa_paleta},
    };
    let mut p = AppConfig::default().profiles.remove(0).paletas.remove(0);
    p.obs = true;
    let mut b = new_button(&p.id, 1, "Camera", "#000", "#fff");
    b.type_field = "obs".into();
    b.obs = Some(ObsAction {
        kind: "scene_program".into(),
        target: "Camera".into(),
        duration_ms: None,
        ..Default::default()
    });
    p.botones.push(b);
    let data = serde_json::to_string(&to_lfa_paleta(&p)).unwrap();
    assert!(!data.contains("password"));
    let q = from_lfa_paleta(serde_json::from_str(&data).unwrap(), "restored".into());
    assert!(q.obs);
    assert_eq!(q.botones[0].obs, p.botones[0].obs);
}

#[test]
fn quick_transitions_only_list_configured_obs_actions_in_numeric_order() {
    let hotkeys = vec![
        "OBSBasic.QuickTransition.12",
        "OBSBasic.StartStreaming",
        "OBSBasic.QuickTransition.2",
        "OBSBasic.QuickTransition.0",
        "OBSBasic.QuickTransition.bad",
    ];
    let hotkeys = hotkeys.into_iter().map(String::from).collect::<Vec<_>>();
    assert_eq!(
        actions::quick_transitions(&hotkeys),
        vec!["OBSBasic.QuickTransition.2", "OBSBasic.QuickTransition.12"]
    );
}
#[test]
fn quick_transition_calls_the_registered_action_and_requires_studio() {
    let a = ObsAction {
        kind: "quick_transition".into(),
        target: "OBSBasic.QuickTransition.12".into(),
        duration_ms: None,
        ..Default::default()
    };
    let mut state = ObsSnapshot {
        studio: true,
        available: vec!["TriggerHotkeyByName".into()],
        quick_transitions: vec![a.target.clone()],
        ..Default::default()
    };
    let (request, data) = actions::request(&a, &state).unwrap();
    assert_eq!(request, "TriggerHotkeyByName");
    assert_eq!(data["hotkeyName"], a.target);
    state.studio = false;
    assert_eq!(
        actions::request(&a, &state).unwrap_err(),
        "obs_studio_required"
    );
    state.studio = true;
    state.quick_transitions.clear();
    assert_eq!(
        actions::request(&a, &state).unwrap_err(),
        "obs_quick_unavailable"
    );
}

#[test]
fn appearance_defaults_and_roundtrips_without_affecting_connection() {
    use crate::model::obs::ObsButtonView;
    let original: ObsConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(original.button_view, ObsButtonView::Border);
    for style in [
        ObsButtonView::Border,
        ObsButtonView::Bottom,
        ObsButtonView::Right,
    ] {
        let mut next = original.clone();
        next.button_view = style;
        let restored: ObsConfig =
            serde_json::from_value(serde_json::to_value(&next).unwrap()).unwrap();
        assert_eq!(restored, next);
        assert!(!connection_changed(&original, &next));
        let mut cfg = AppConfig::default();
        cfg.obs = next;
        assert_eq!(view::view(&cfg, ObsSnapshot::default()).button_view, style);
    }
    assert!(serde_json::from_str::<ObsConfig>(r#"{"button_view":"unknown"}"#).is_err());
}
#[test]
fn connection_changes_are_not_confused_with_appearance() {
    let original = ObsConfig::default();
    for field in ["enabled", "host", "port", "reconnect"] {
        let mut next = original.clone();
        match field {
            "enabled" => next.enabled = true,
            "host" => next.host = "obs.local".into(),
            "port" => next.port = 4456,
            _ => next.reconnect = false,
        }
        assert!(connection_changed(&original, &next));
    }
}
