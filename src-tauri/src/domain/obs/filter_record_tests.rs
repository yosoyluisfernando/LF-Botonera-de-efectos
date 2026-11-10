use super::{actions, button_state::action_state};
use crate::model::obs::{ObsAction, ObsFilter, ObsSnapshot};

fn action(kind: &str) -> ObsAction {
    ObsAction {
        kind: kind.into(),
        ..Default::default()
    }
}
#[test]
fn filter_toggle_and_independent_actions_target_the_source_and_filter() {
    let mut s = ObsSnapshot {
        available: vec!["SetSourceFilterEnabled".into()],
        filters: vec![
            ObsFilter {
                source: "Camera".into(),
                name: "Color".into(),
                enabled: false,
            },
            ObsFilter {
                source: "Scene".into(),
                name: "Color".into(),
                enabled: true,
            },
        ],
        ..Default::default()
    };
    for enabled in [false, true] {
        s.filters[0].enabled = enabled;
        for (kind, expected) in [
            ("filter_on", true),
            ("filter_off", false),
            ("filter_toggle", !enabled),
        ] {
            let a = ObsAction {
                target: "Camera".into(),
                filter: "Color".into(),
                ..action(kind)
            };
            let (method, data) = actions::request(&a, &s).unwrap();
            assert_eq!(method, "SetSourceFilterEnabled");
            assert_eq!(data["sourceName"], "Camera");
            assert_eq!(data["filterName"], "Color");
            assert_eq!(data["filterEnabled"], expected);
            assert_eq!(
                action_state(&a, &s),
                if enabled { "filter_on" } else { "filter_off" }
            );
        }
    }
    let a = ObsAction {
        target: "Missing".into(),
        filter: "Color".into(),
        ..action("filter_toggle")
    };
    assert_eq!(
        actions::request(&a, &s).unwrap_err(),
        "obs_target_unavailable"
    );
    assert_eq!(action_state(&a, &s), "unavailable");
}
#[test]
fn pause_resume_and_split_do_not_replace_start_stop_and_check_real_state() {
    let mut s = ObsSnapshot {
        available: [
            "PauseRecord",
            "ResumeRecord",
            "SplitRecordFile",
            "StopRecord",
        ]
        .map(String::from)
        .to_vec(),
        ..Default::default()
    };
    for kind in [
        "record_pause_toggle",
        "record_pause",
        "record_resume",
        "record_split",
    ] {
        assert_eq!(
            actions::request(&action(kind), &s).unwrap_err(),
            "obs_record_inactive"
        );
        assert_eq!(action_state(&action(kind), &s), "recording_stopped");
    }
    s.recording = true;
    assert_eq!(
        actions::request(&action("record_pause_toggle"), &s)
            .unwrap()
            .0,
        "PauseRecord"
    );
    assert_eq!(
        actions::request(&action("record_split"), &s).unwrap().0,
        "SplitRecordFile"
    );
    assert_eq!(action_state(&action("record_split"), &s), "split_ready");
    s.recording_paused = true;
    assert_eq!(
        actions::request(&action("record_pause_toggle"), &s)
            .unwrap()
            .0,
        "ResumeRecord"
    );
    assert_eq!(
        actions::request(&action("record_toggle"), &s).unwrap().0,
        "StopRecord"
    );
    assert_eq!(
        actions::request(&action("record_split"), &s).unwrap_err(),
        "obs_record_split_paused"
    );
    for kind in ["record_toggle", "record_stop", "record_pause_toggle"] {
        assert_eq!(action_state(&action(kind), &s), "recording_paused");
    }
    s.available.clear();
    assert_eq!(
        actions::request(&action("record_resume"), &s).unwrap_err(),
        "obs_unsupported"
    );
    assert_eq!(action_state(&action("record_resume"), &s), "unavailable");
}
#[test]
fn kinds_are_unique_and_new_identity_defaults_validate_without_mixing() {
    let unique = actions::KINDS
        .iter()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), actions::KINDS.len());
    for a in [
        action("filter_on"),
        ObsAction {
            target: "Camera".into(),
            ..action("filter_toggle")
        },
        ObsAction {
            filter: "Color".into(),
            ..action("record_pause")
        },
        ObsAction {
            duration_ms: Some(300),
            ..action("record_split")
        },
    ] {
        assert!(actions::validate(&a).is_err());
    }
    let old: ObsAction = serde_json::from_str(r#"{"kind":"record_toggle"}"#).unwrap();
    assert!(old.filter.is_empty());
    assert!(actions::validate(&old).is_ok());
    let a = ObsAction {
        target: "Camera".into(),
        filter: "Color".into(),
        ..action("filter_toggle")
    };
    assert_eq!(
        serde_json::from_value::<ObsAction>(serde_json::to_value(&a).unwrap()).unwrap(),
        a
    );
}
