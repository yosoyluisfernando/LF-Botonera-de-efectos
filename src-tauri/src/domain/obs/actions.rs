use crate::model::obs::{ObsAction, ObsSnapshot};
use serde_json::{json, Value};

pub const KINDS: &[&str] = &[
    "scene_program",
    "scene_preview",
    "transition",
    "quick_transition",
    "transition_select",
    "studio_toggle",
    "studio_on",
    "studio_off",
    "record_toggle",
    "record_start",
    "record_stop",
    "record_pause_toggle",
    "record_pause",
    "record_resume",
    "record_split",
    "stream_toggle",
    "stream_start",
    "stream_stop",
    "input_toggle",
    "input_mute",
    "input_unmute",
    "item_toggle",
    "item_show",
    "item_hide",
    "replay_toggle",
    "replay_start",
    "replay_stop",
    "replay_save",
    "filter_toggle",
    "filter_on",
    "filter_off",
];
pub fn validate(a: &ObsAction) -> Result<(), String> {
    super::filter_record::validate(a)?;
    super::resource_actions::validate(a)?;
    if (a.kind == "quick_transition"
        && (quick_transition_id(&a.target).is_none() || a.duration_ms.is_some()))
        || !KINDS.contains(&a.kind.as_str())
        || a.target.len() > 1024
        || (matches!(
            a.kind.as_str(),
            "scene_program" | "scene_preview" | "transition_select"
        ) && a.target.is_empty())
        || a.duration_ms.is_some_and(|v| !(50..=20000).contains(&v))
    {
        return Err("obs_invalid_action".into());
    }
    Ok(())
}
pub fn request(a: &ObsAction, s: &ObsSnapshot) -> Result<(&'static str, Value), String> {
    validate(a)?;
    let (name, data) = match a.kind.as_str() {
        "scene_program" => ("SetCurrentProgramScene", json!({"sceneName":a.target})),
        "scene_preview" => {
            if !s.studio {
                return Err("obs_studio_required".into());
            }
            ("SetCurrentPreviewScene", json!({"sceneName":a.target}))
        }
        "transition" => {
            if !s.studio {
                return Err("obs_studio_required".into());
            }
            ("TriggerStudioModeTransition", json!({}))
        }
        "quick_transition" => {
            if !s.studio {
                return Err("obs_studio_required".into());
            }
            if !s.quick_transitions.contains(&a.target) {
                return Err("obs_quick_unavailable".into());
            }
            ("TriggerHotkeyByName", json!({"hotkeyName":a.target}))
        }
        "transition_select" => (
            "SetCurrentSceneTransition",
            json!({"transitionName":a.target}),
        ),
        "studio_on" => ("SetStudioModeEnabled", json!({"studioModeEnabled":true})),
        "studio_off" => ("SetStudioModeEnabled", json!({"studioModeEnabled":false})),
        "studio_toggle" => (
            "SetStudioModeEnabled",
            json!({"studioModeEnabled":!s.studio}),
        ),
        "record_start" => ("StartRecord", json!({})),
        "record_stop" => ("StopRecord", json!({})),
        "record_toggle" => (
            if s.recording {
                "StopRecord"
            } else {
                "StartRecord"
            },
            json!({}),
        ),
        "stream_start" => ("StartStream", json!({})),
        "stream_stop" => ("StopStream", json!({})),
        "stream_toggle" => (
            if s.streaming {
                "StopStream"
            } else {
                "StartStream"
            },
            json!({}),
        ),
        kind if super::filter_record::handles(kind) => super::filter_record::request(a, s)?,
        _ => super::resource_actions::request(a, s)?,
    };
    if !s.available.iter().any(|n| n == name) {
        return Err("obs_unsupported".into());
    }
    Ok((name, data))
}

/// Solo acciones rápidas reales; no otros atajos ni números supuestos.
pub fn quick_transition_id(name: &str) -> Option<u32> {
    let id = name.strip_prefix("OBSBasic.QuickTransition.")?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    id.parse::<u32>().ok().filter(|id| *id > 0)
}
pub fn quick_transitions(hotkeys: &[String]) -> Vec<String> {
    let mut found = hotkeys
        .iter()
        .filter(|name| quick_transition_id(name).is_some())
        .cloned()
        .collect::<Vec<_>>();
    found.sort_by_key(|name| quick_transition_id(name));
    found.dedup();
    found
}
