use super::{actions, button_state::action_state};
use crate::model::obs::{ObsAction, ObsInput, ObsSceneItem, ObsSnapshot};

fn action(kind: &str) -> ObsAction {
    ObsAction {
        kind: kind.into(),
        ..Default::default()
    }
}
#[test]
fn mute_controls_follow_real_input_state_and_keep_independent_actions() {
    let mut state = ObsSnapshot {
        available: vec!["SetInputMute".into()],
        inputs: vec![ObsInput {
            name: "Mic".into(),
            muted: false,
        }],
        ..Default::default()
    };
    for muted in [false, true] {
        state.inputs[0].muted = muted;
        for (kind, expected) in [
            ("input_toggle", !muted),
            ("input_mute", true),
            ("input_unmute", false),
        ] {
            let mut a = action(kind);
            a.target = "Mic".into();
            let (method, data) = actions::request(&a, &state).unwrap();
            assert_eq!(method, "SetInputMute");
            assert_eq!(data["inputName"], "Mic");
            assert_eq!(data["inputMuted"], expected);
            assert_eq!(
                action_state(&a, &state),
                if muted { "input_muted" } else { "input_live" }
            );
        }
    }
    let mut a = action("input_toggle");
    a.target = "Missing".into();
    assert_eq!(
        actions::request(&a, &state).unwrap_err(),
        "obs_target_unavailable"
    );
    assert_eq!(action_state(&a, &state), "unavailable");
}
#[test]
fn visibility_uses_container_id_and_name_and_never_retargets_reused_ids() {
    let mut state = ObsSnapshot {
        available: vec!["SetSceneItemEnabled".into()],
        scene_items: vec![
            ObsSceneItem {
                scene: "Main".into(),
                id: 2,
                name: "Logo".into(),
                enabled: true,
            },
            ObsSceneItem {
                scene: "Group".into(),
                id: 2,
                name: "Logo".into(),
                enabled: false,
            },
        ],
        ..Default::default()
    };
    for (kind, enabled) in [
        ("item_toggle", true),
        ("item_show", true),
        ("item_hide", false),
    ] {
        let a = ObsAction {
            target: "Logo".into(),
            scene: "Group".into(),
            scene_item_id: Some(2),
            ..action(kind)
        };
        let (_, data) = actions::request(&a, &state).unwrap();
        assert_eq!(data["sceneName"], "Group");
        assert_eq!(data["sceneItemId"], 2);
        assert_eq!(data["sceneItemEnabled"], enabled);
        assert_eq!(action_state(&a, &state), "item_hidden");
        state.scene_items[1].enabled = true;
        assert_eq!(action_state(&a, &state), "item_visible");
        state.scene_items[1].enabled = false;
    }
    let a = ObsAction {
        target: "Old camera".into(),
        scene: "Main".into(),
        scene_item_id: Some(2),
        ..action("item_toggle")
    };
    assert_eq!(
        actions::request(&a, &state).unwrap_err(),
        "obs_target_unavailable"
    );
    assert_eq!(action_state(&a, &state), "unavailable");
}
#[test]
fn replay_distinguishes_unavailable_stopped_running_and_missing_capability() {
    let mut state = ObsSnapshot::default();
    let save = action("replay_save");
    assert_eq!(
        actions::request(&save, &state).unwrap_err(),
        "obs_replay_unavailable"
    );
    assert_eq!(action_state(&save, &state), "replay_unavailable");
    state.replay_active = Some(false);
    assert_eq!(
        actions::request(&save, &state).unwrap_err(),
        "obs_replay_inactive"
    );
    assert_eq!(action_state(&save, &state), "replay_stopped");
    state.available = ["StartReplayBuffer", "StopReplayBuffer", "SaveReplayBuffer"]
        .map(String::from)
        .to_vec();
    assert_eq!(
        actions::request(&action("replay_toggle"), &state)
            .unwrap()
            .0,
        "StartReplayBuffer"
    );
    state.replay_active = Some(true);
    assert_eq!(
        actions::request(&action("replay_toggle"), &state)
            .unwrap()
            .0,
        "StopReplayBuffer"
    );
    assert_eq!(
        actions::request(&save, &state).unwrap().0,
        "SaveReplayBuffer"
    );
    assert_eq!(action_state(&save, &state), "replay_ready");
    assert_eq!(
        action_state(&action("replay_stop"), &state),
        "replay_running"
    );
    state.available.clear();
    assert_eq!(
        actions::request(&save, &state).unwrap_err(),
        "obs_unsupported"
    );
}
#[test]
fn resource_actions_reject_incomplete_or_mixed_identities() {
    for a in [
        action("input_mute"),
        action("item_show"),
        ObsAction {
            target: "Logo".into(),
            scene: "Main".into(),
            scene_item_id: Some(-1),
            ..action("item_toggle")
        },
        ObsAction {
            target: "Mic".into(),
            scene_item_id: Some(2),
            ..action("input_toggle")
        },
        ObsAction {
            target: "Mic".into(),
            duration_ms: Some(300),
            ..action("input_mute")
        },
        ObsAction {
            target: "Unexpected".into(),
            ..action("replay_save")
        },
    ] {
        assert!(actions::validate(&a).is_err());
    }
}
