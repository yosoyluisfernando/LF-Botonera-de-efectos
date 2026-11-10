//! Datos persistidos de OBS y estado operativo independiente del audio.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ObsButtonView {
    #[default]
    Border,
    Bottom,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ObsConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub reconnect: bool,
    #[serde(default)]
    pub button_view: ObsButtonView,
}
impl Default for ObsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".into(),
            port: 4455,
            reconnect: true,
            button_view: ObsButtonView::default(),
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ObsAction {
    pub kind: String,
    pub target: String,
    pub duration_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scene: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_item_id: Option<i64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub filter: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ObsFilter {
    pub source: String,
    pub name: String,
    pub enabled: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct ObsInput {
    pub name: String,
    pub muted: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct ObsSceneItem {
    pub scene: String,
    pub id: i64,
    pub name: String,
    pub enabled: bool,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct ObsSnapshot {
    pub status: String,
    pub error: String,
    pub connected: bool,
    pub busy: bool,
    pub pending_id: String,
    pub failed_id: String,
    pub studio: bool,
    pub program: String,
    pub preview: String,
    pub recording: bool,
    pub recording_paused: bool,
    pub filters: Vec<ObsFilter>,
    pub streaming: bool,
    pub replay_active: Option<bool>,
    pub inputs: Vec<ObsInput>,
    pub scene_items: Vec<ObsSceneItem>,
    pub transition: String,
    pub fixed_duration: bool,
    pub scenes: Vec<String>,
    pub transitions: Vec<String>,
    pub quick_transitions: Vec<String>,
    pub available: Vec<String>,
}
