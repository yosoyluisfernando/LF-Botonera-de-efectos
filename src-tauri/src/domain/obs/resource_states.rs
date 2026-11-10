use super::resource_actions::{is_input, is_item, item};
use crate::model::obs::{ObsAction, ObsSnapshot};

pub fn state(a: &ObsAction, s: &ObsSnapshot) -> &'static str {
    if is_input(&a.kind) {
        return match s.inputs.iter().find(|i| i.name == a.target) {
            Some(i) if i.muted => "input_muted",
            Some(_) => "input_live",
            None => "unavailable",
        };
    }
    if is_item(&a.kind) {
        return match item(a, s) {
            Some(i) if i.enabled => "item_visible",
            Some(_) => "item_hidden",
            None => "unavailable",
        };
    }
    if a.kind.starts_with("replay_") {
        return match s.replay_active {
            Some(true) if a.kind == "replay_save" => "replay_ready",
            Some(true) => "replay_running",
            Some(false) => "replay_stopped",
            None => "replay_unavailable",
        };
    }
    "ready"
}
