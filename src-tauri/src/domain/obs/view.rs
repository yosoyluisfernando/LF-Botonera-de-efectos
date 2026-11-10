use crate::model::{obs::ObsSnapshot, AppConfig};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ObsView {
    #[serde(flatten)]
    pub state: ObsSnapshot,
    pub buttons: Vec<ButtonStatus>,
    pub actions: Vec<ActionOption>,
    pub button_view: crate::model::obs::ObsButtonView,
}
#[derive(Clone, Serialize)]
pub struct ActionOption {
    pub kind: &'static str,
    pub target: &'static str,
    pub duration: bool,
}
#[derive(Clone, Serialize)]
pub struct ButtonStatus {
    pub id: String,
    pub state: &'static str,
    pub error: String,
}
pub fn view(cfg: &AppConfig, state: ObsSnapshot) -> ObsView {
    let buttons = cfg
        .active_profile()
        .into_iter()
        .flat_map(|p| &p.paletas)
        .filter(|p| p.obs)
        .flat_map(|p| &p.botones)
        .map(|b| {
            let status = if !cfg.obs.enabled {
                "disabled"
            } else if !state.connected {
                "disconnected"
            } else if state.pending_id == b.id {
                "pending"
            } else if state.failed_id == b.id {
                "error"
            } else {
                b.obs
                    .as_ref()
                    .map(|a| super::button_state::action_state(a, &state))
                    .unwrap_or("ready")
            };
            ButtonStatus {
                id: b.id.clone(),
                state: status,
                error: if state.failed_id == b.id {
                    state.error.clone()
                } else {
                    String::new()
                },
            }
        })
        .collect();
    let actions = super::actions::KINDS
        .iter()
        .map(|kind| ActionOption {
            kind,
            target: match *kind {
                "scene_program" | "scene_preview" => "scenes",
                "transition_select" => "transitions",
                "quick_transition" => "quick_transitions",
                "filter_toggle" | "filter_on" | "filter_off" => "filters",
                kind if super::resource_actions::is_input(kind) => "inputs",
                kind if super::resource_actions::is_item(kind) => "scene_items",
                _ => "",
            },
            duration: *kind == "transition_select",
        })
        .collect();
    ObsView {
        state,
        buttons,
        actions,
        button_view: cfg.obs.button_view,
    }
}
