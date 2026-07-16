use super::ButtonData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixedPanelConfig {
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default = "default_view")]
    pub view: String,
    #[serde(default = "default_side")]
    pub side: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_true")]
    pub show_on_start: bool,
    #[serde(default = "default_columns")]
    pub columns: u32,
    #[serde(default = "default_row_mode")]
    pub row_mode: String,
    #[serde(default = "default_rows")]
    pub rows: u32,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default)]
    pub global_buttons: Vec<ButtonData>,
    #[serde(default = "default_playback_mode")]
    pub playback_mode: String,
    #[serde(default)]
    pub solo_mode: bool,
    #[serde(default = "default_modes_position")]
    pub modes_position: String,
}

impl Default for FixedPanelConfig {
    fn default() -> Self {
        Self {
            scope: default_scope(),
            view: default_view(),
            side: default_side(),
            visible: true,
            show_on_start: true,
            columns: default_columns(), row_mode: default_row_mode(),
            rows: default_rows(), width: default_width(),
            global_buttons: Vec::new(),
            playback_mode: default_playback_mode(),
            solo_mode: false,
            modes_position: default_modes_position(),
        }
    }
}

fn default_scope() -> String {
    "global".into()
}
fn default_view() -> String {
    "player".into()
}
fn default_side() -> String {
    "right".into()
}
fn default_true() -> bool {
    true
}
fn default_columns() -> u32 {
    1
}
fn default_row_mode() -> String { "unlimited".into() }
fn default_rows() -> u32 { 10 }
fn default_width() -> u32 { 240 }
fn default_playback_mode() -> String {
    "normal".into()
}
fn default_modes_position() -> String { "top".into() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_product_decisions() {
        let panel = FixedPanelConfig::default();
        assert_eq!(panel.scope, "global");
        assert_eq!(panel.view, "player");
        assert_eq!(panel.side, "right");
        assert_eq!(panel.columns, 1);
        assert_eq!(panel.row_mode, "unlimited");
        assert_eq!(panel.rows, 10);
        assert!(panel.show_on_start);
    }
}
