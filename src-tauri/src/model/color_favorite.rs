//! Combinaciones de colores globales; independientes de perfiles y sonidos.
use crate::model::ButtonStyle;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorFavorite {
    pub name: String,
    pub color_bg: String,
    pub color_text: String,
    #[serde(default, skip_serializing_if = "ButtonStyle::is_default")]
    pub style: ButtonStyle,
}
impl ColorFavorite {
    pub fn normalized(mut self) -> Result<Self, String> {
        self.name = self.name.trim().to_string();
        if self.name.is_empty()
            || self.name.chars().count() > 80
            || self.name.chars().any(char::is_control)
        {
            return Err("color_favorite_name_invalid".into());
        }
        self.style
            .validate_colors(&self.color_bg, &self.color_text)?;
        self.color_bg.make_ascii_uppercase();
        self.color_text.make_ascii_uppercase();
        self.style.bg2.make_ascii_uppercase();
        self.style.text2.make_ascii_uppercase();
        Ok(self)
    }
}
pub fn add(list: &mut Vec<ColorFavorite>, favorite: ColorFavorite) -> Result<(), String> {
    let value = favorite.normalized()?;
    if list
        .iter()
        .any(|f| f.name.to_lowercase() == value.name.to_lowercase())
    {
        return Err("color_favorite_exists".into());
    }
    list.push(value);
    list.sort_by_key(|f| f.name.to_lowercase());
    Ok(())
}
pub fn remove(list: &mut Vec<ColorFavorite>, name: &str) -> Result<(), String> {
    let position = list
        .iter()
        .position(|f| f.name == name)
        .ok_or("color_favorite_not_found")?;
    list.remove(position);
    Ok(())
}
#[cfg(test)]
#[path = "color_favorite_tests.rs"]
mod tests;
