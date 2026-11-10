use crate::model::obs::{ObsAction, ObsSnapshot};

pub(super) fn action_state(a: &ObsAction, s: &ObsSnapshot) -> &'static str {
    if super::filter_record::handles(&a.kind) {
        return super::filter_record::state(a, s);
    }
    match a.kind.as_str() {
        "scene_program" | "scene_preview" if s.program == a.target => "program",
        "scene_program" | "scene_preview" if s.studio && s.preview == a.target => "preview",
        "studio_on" | "studio_off" | "studio_toggle" => {
            if s.studio {
                "studio_on"
            } else {
                "studio_off"
            }
        }
        "record_start" | "record_stop" | "record_toggle" => {
            if s.recording && s.recording_paused {
                "recording_paused"
            } else if s.recording {
                "recording"
            } else {
                "recording_stopped"
            }
        }
        "stream_start" | "stream_stop" | "stream_toggle" => {
            if s.streaming {
                "streaming"
            } else {
                "streaming_stopped"
            }
        }
        "transition_select" if s.transition == a.target => "active",
        _ => super::resource_states::state(a, s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_buttons_show_actual_function_state_including_stop_only_actions() {
        let mut state = ObsSnapshot::default();
        for (kinds, off, on) in [
            (
                ["studio_on", "studio_off", "studio_toggle"],
                "studio_off",
                "studio_on",
            ),
            (
                ["record_start", "record_stop", "record_toggle"],
                "recording_stopped",
                "recording",
            ),
            (
                ["stream_start", "stream_stop", "stream_toggle"],
                "streaming_stopped",
                "streaming",
            ),
        ] {
            state.studio = false;
            state.recording = false;
            state.streaming = false;
            for kind in kinds {
                let action = ObsAction {
                    kind: kind.into(),
                    ..Default::default()
                };
                assert_eq!(action_state(&action, &state), off);
            }
            state.studio = true;
            state.recording = true;
            state.streaming = true;
            for kind in kinds {
                let action = ObsAction {
                    kind: kind.into(),
                    ..Default::default()
                };
                assert_eq!(action_state(&action, &state), on);
            }
        }
    }
    #[test]
    fn recording_does_not_light_up_studio_or_streaming() {
        let mut state = ObsSnapshot {
            recording: true,
            ..Default::default()
        };
        let action = |kind: &str| ObsAction {
            kind: kind.into(),
            ..Default::default()
        };
        assert_eq!(action_state(&action("record_toggle"), &state), "recording");
        assert_eq!(action_state(&action("studio_toggle"), &state), "studio_off");
        assert_eq!(
            action_state(&action("stream_toggle"), &state),
            "streaming_stopped"
        );
        state.recording = false;
        state.streaming = true;
        assert_eq!(
            action_state(&action("record_toggle"), &state),
            "recording_stopped"
        );
        assert_eq!(action_state(&action("stream_toggle"), &state), "streaming");
    }
    #[test]
    fn scene_program_and_preview_remain_distinct_from_transport_states() {
        let state = ObsSnapshot {
            studio: true,
            program: "Camera".into(),
            preview: "Desktop".into(),
            ..Default::default()
        };
        for kind in ["scene_program", "scene_preview"] {
            let mut action = ObsAction {
                kind: kind.into(),
                target: "Camera".into(),
                ..Default::default()
            };
            assert_eq!(action_state(&action, &state), "program");
            action.target = "Desktop".into();
            assert_eq!(action_state(&action, &state), "preview");
        }
    }
}
