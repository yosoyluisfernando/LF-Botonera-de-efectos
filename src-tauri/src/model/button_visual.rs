//! Identificador visual persistente de un botón.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ButtonVisual {
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub value: String,
    #[serde(default = "default_mode")]
    pub mode: String,
}

impl Default for ButtonVisual {
    fn default() -> Self {
        Self {
            kind: default_kind(),
            value: String::new(),
            mode: default_mode(),
        }
    }
}

impl ButtonVisual {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

fn default_kind() -> String {
    "auto".to_string()
}

fn default_mode() -> String {
    "visual_text".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ButtonData;

    #[test]
    fn missing_fields_reproduce_the_existing_presentation() {
        let visual: ButtonVisual = serde_json::from_str("{}").unwrap();
        assert_eq!(visual, ButtonVisual::default());
        assert_eq!(visual.kind, "auto");
        assert_eq!(visual.mode, "visual_text");
    }

    #[test]
    fn old_buttons_load_without_adding_default_json() {
        let source = r##"{
            "id":"paleta_1_btn_0","index":0,"label":"Aplausos",
            "color_bg":"#000000","color_text":"#ffffff"
        }"##;
        let button: ButtonData = serde_json::from_str(source).unwrap();
        assert_eq!(button.visual, ButtonVisual::default());
        let saved = serde_json::to_value(button).unwrap();
        assert!(saved.get("visual").is_none());
    }

    #[test]
    fn custom_visual_is_serialized() {
        let visual = ButtonVisual {
            kind: "emoji".into(),
            value: "👏".into(),
            mode: "visual".into(),
        };
        assert!(!visual.is_default());
        assert_eq!(serde_json::to_value(visual).unwrap()["value"], "👏");
    }
}
