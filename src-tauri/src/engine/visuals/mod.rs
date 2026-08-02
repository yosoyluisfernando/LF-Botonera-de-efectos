mod category_terms;
mod data;
mod search;
mod types;

pub use search::{basic_groups, groups, groups_visible, search, search_basics, search_visible};
pub use types::{VisualCatalogPage, VisualGroup};

use crate::model::ButtonVisual;

pub fn validate_button_visual(visual: &ButtonVisual) -> Result<(), String> {
    if !matches!(visual.mode.as_str(), "text" | "visual_text" | "visual") {
        return Err("invalid_visual_mode".to_string());
    }
    match visual.kind.as_str() {
        "auto" if visual.value.is_empty() => Ok(()),
        "emoji" if !visual.value.is_empty() && data::contains_value(&visual.value)? => Ok(()),
        "basic" if !visual.value.is_empty() && data::contains_basic_value(&visual.value)? => Ok(()),
        "basic" => Err("invalid_visual_value".to_string()),
        "auto" | "emoji" => Err("invalid_visual_value".to_string()),
        _ => Err("invalid_visual_kind".to_string()),
    }
}

#[cfg(test)]
mod tests;
