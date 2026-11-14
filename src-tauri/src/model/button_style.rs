//! Estilos de color adaptados de Danilo Wenner (7c1f1ae), sin tamaño de fuente.
use serde::{Deserialize, Serialize};
pub const GRADIENTS: [&str; 5] = ["", "horizontal", "vertical", "diagonal", "diagonal_inv"];

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(try_from = "StyleFields")]
pub struct ButtonStyle {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gradient: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bg2: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text2: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub exact: bool,
}
#[derive(Deserialize, Default)]
#[serde(default)]
struct StyleFields {
    gradient: String,
    bg2: String,
    text2: String,
    exact: bool,
}
impl TryFrom<StyleFields> for ButtonStyle {
    type Error = String;
    fn try_from(raw: StyleFields) -> Result<Self, String> {
        let value = Self {
            gradient: raw.gradient,
            bg2: raw.bg2,
            text2: raw.text2,
            exact: raw.exact,
        };
        value.validate()?;
        Ok(value)
    }
}
fn is_false(v: &bool) -> bool {
    !*v
}
impl ButtonStyle {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn validate(&self) -> Result<(), String> {
        if !GRADIENTS.contains(&self.gradient.as_str())
            || (!self.gradient.is_empty() && self.bg2.is_empty())
        {
            return Err("invalid_gradient".into());
        }
        for color in [&self.bg2, &self.text2] {
            if !color.is_empty() && !is_hex_color(color) {
                return Err("invalid_hex_color".into());
            }
        }
        Ok(())
    }
    pub fn validate_colors(&self, bg: &str, text: &str) -> Result<(), String> {
        self.validate()?;
        if !is_hex_color(bg) || !is_hex_color(text) {
            return Err("invalid_hex_color".into());
        }
        Ok(())
    }
}
pub fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit)
}
#[cfg(test)]
#[path = "button_style_tests.rs"]
mod tests;
