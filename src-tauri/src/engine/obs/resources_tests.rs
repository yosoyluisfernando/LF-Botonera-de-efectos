use super::super::test_support::mock;
use super::*;

#[test]
fn catalogs_filter_non_audio_keep_group_identities_and_tolerate_disabled_replay() {
    let (mut session, server) = mock(vec![
        (
            "GetInputList",
            json!({}),
            Ok(json!({"inputs":[{"inputName":"Mic"},{"inputName":"Image"}]})),
        ),
        (
            "GetInputMute",
            json!({"inputName":"Mic"}),
            Ok(json!({"inputMuted":true})),
        ),
        ("GetInputMute", json!({"inputName":"Image"}), Err(604)),
        (
            "GetSceneItemList",
            json!({"sceneName":"Main"}),
            Ok(json!({"sceneItems":[
                {"sceneItemId":3,"sourceName":"Logo","sceneItemEnabled":true,"sceneItemIndex":0},
                {"sceneItemId":4,"sourceName":"Group","sceneItemEnabled":false,"sceneItemIndex":1,"isGroup":true}
            ]})),
        ),
        (
            "GetGroupSceneItemList",
            json!({"sceneName":"Group"}),
            Ok(json!({"sceneItems":[
                {"sceneItemId":3,"sourceName":"Logo","sceneItemEnabled":false,"sceneItemIndex":0}
            ]})),
        ),
        ("GetReplayBufferStatus", json!({}), Err(604)),
    ]);
    let mut state = ObsSnapshot {
        scenes: vec!["Main".into()],
        available: [
            "GetInputList",
            "GetInputMute",
            "SetInputMute",
            "GetSceneItemList",
            "GetGroupSceneItemList",
            "SetSceneItemEnabled",
            "GetReplayBufferStatus",
        ]
        .map(String::from)
        .to_vec(),
        ..Default::default()
    };
    refresh(&mut session, &mut state).unwrap();
    assert_eq!(state.inputs.len(), 1);
    assert!(state.inputs[0].muted);
    assert_eq!(state.scene_items.len(), 3);
    assert_eq!(state.scene_items[0].name, "Group");
    assert_eq!(state.scene_items[2].scene, "Group");
    assert!(!state.scene_items[2].enabled);
    assert_eq!(state.replay_active, None);
    server.join().unwrap();
}
#[test]
fn unexpected_obs_error_is_not_treated_as_non_audio_or_replay_stopped() {
    let (mut session, server) = mock(vec![("GetReplayBufferStatus", json!({}), Err(205))]);
    let mut state = ObsSnapshot {
        available: vec!["GetReplayBufferStatus".into()],
        ..Default::default()
    };
    assert_eq!(
        refresh(&mut session, &mut state).unwrap_err(),
        "obs_request_failed"
    );
    server.join().unwrap();
}
#[test]
fn malformed_items_are_not_assumed_hidden_and_absent_resources_clear_stale_state() {
    assert!(parse_item("Main", &json!({"sceneItemId":1,"sourceName":"Logo"})).is_err());
    assert!(parse_item(
        "Main",
        &json!({"sceneItemId":-1,"sourceName":"Logo","sceneItemEnabled":false})
    )
    .is_err());
    let (mut session, server) = mock(vec![]);
    let mut state = ObsSnapshot {
        replay_active: Some(true),
        inputs: vec![ObsInput {
            name: "Old".into(),
            muted: true,
        }],
        ..Default::default()
    };
    refresh(&mut session, &mut state).unwrap();
    assert!(state.inputs.is_empty());
    assert_eq!(state.replay_active, None);
    server.join().unwrap();
}
