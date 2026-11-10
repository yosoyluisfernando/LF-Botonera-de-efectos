//! Identidad y órdenes de audio, elementos de escena y búfer; sin I/O.
use crate::model::obs::{ObsAction, ObsSceneItem, ObsSnapshot};
use serde_json::{json, Value};

pub fn is_input(kind: &str) -> bool {
    matches!(kind, "input_toggle" | "input_mute" | "input_unmute")
}
pub fn is_item(kind: &str) -> bool {
    matches!(kind, "item_toggle" | "item_show" | "item_hide")
}
pub fn validate(a: &ObsAction) -> Result<(), String> {
    if ((is_input(&a.kind) || is_item(&a.kind)) && (a.target.is_empty() || a.duration_ms.is_some()))
        || a.scene.len() > 1024
        || (is_item(&a.kind) && (a.scene.is_empty() || a.scene_item_id.is_none_or(|id| id < 0)))
        || (!is_item(&a.kind) && (!a.scene.is_empty() || a.scene_item_id.is_some()))
        || (a.kind.starts_with("replay_") && (!a.target.is_empty() || a.duration_ms.is_some()))
    {
        return Err("obs_invalid_action".into());
    }
    Ok(())
}
pub fn item<'a>(a: &ObsAction, s: &'a ObsSnapshot) -> Option<&'a ObsSceneItem> {
    s.scene_items.iter().find(|item| {
        item.scene == a.scene && Some(item.id) == a.scene_item_id && item.name == a.target
    })
}
pub fn request(a: &ObsAction, s: &ObsSnapshot) -> Result<(&'static str, Value), String> {
    if is_input(&a.kind) {
        let input = s
            .inputs
            .iter()
            .find(|i| i.name == a.target)
            .ok_or("obs_target_unavailable")?;
        let muted = match a.kind.as_str() {
            "input_mute" => true,
            "input_unmute" => false,
            _ => !input.muted,
        };
        return Ok((
            "SetInputMute",
            json!({"inputName":a.target,"inputMuted":muted}),
        ));
    }
    if is_item(&a.kind) {
        let item = item(a, s).ok_or("obs_target_unavailable")?;
        let enabled = match a.kind.as_str() {
            "item_show" => true,
            "item_hide" => false,
            _ => !item.enabled,
        };
        return Ok((
            "SetSceneItemEnabled",
            json!({
                "sceneName":a.scene,"sceneItemId":item.id,"sceneItemEnabled":enabled
            }),
        ));
    }
    let active = s.replay_active.ok_or("obs_replay_unavailable")?;
    let name = match a.kind.as_str() {
        "replay_start" => "StartReplayBuffer",
        "replay_stop" => "StopReplayBuffer",
        "replay_toggle" => {
            if active {
                "StopReplayBuffer"
            } else {
                "StartReplayBuffer"
            }
        }
        "replay_save" if active => "SaveReplayBuffer",
        "replay_save" => return Err("obs_replay_inactive".into()),
        _ => return Err("obs_invalid_action".into()),
    };
    Ok((name, json!({})))
}
