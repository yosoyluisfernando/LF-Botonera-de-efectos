//! Modulo: model/button_style.rs
//! Proposito: la presentacion avanzada de un boton: tamaño de texto propio,
//! degradados de dos colores y color exacto. Todo opcional y omitido del JSON
//! mientras valga lo de siempre: un archivo antiguo se ve exactamente igual y el
//! LF Automatizador ignora el campo entero (regla 6).
use serde::{Deserialize, Serialize};

/// Tamaños de texto admitidos, en px. 0 significa "el de la pestaña".
pub const FONT_MIN: u32 = 8;
pub const FONT_MAX: u32 = 48;

/// Direcciones de degradado. "" = color solido.
pub const GRADIENTS: [&str; 5] = ["", "horizontal", "vertical", "diagonal", "diagonal_inv"];

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ButtonStyle {
    /// Tamaño de texto propio del boton. 0 = hereda el de su pestaña.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub font_size: u32,
    /// Direccion del degradado del fondo y del texto. "" = solido.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gradient: String,
    /// Segundo color del fondo (con degradado).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bg2: String,
    /// Segundo color del texto. Vacio = texto de un solo color.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text2: String,
    /// Pintar los colores TAL CUAL, sin adaptarlos al tema. Lo activa el editor
    /// al elegir un color propio: quien escribe #FF0000 quiere ver rojo puro.
    #[serde(default, skip_serializing_if = "is_false")]
    pub exact: bool,
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}
fn is_false(v: &bool) -> bool {
    !*v
}

impl ButtonStyle {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Rechaza lo que la UI no puede pintar. Un error aqui es del llamante.
    pub fn validate(&self) -> Result<(), String> {
        validate_font_size(self.font_size)?;
        if !GRADIENTS.contains(&self.gradient.as_str()) {
            return Err("invalid_gradient".into());
        }
        for color in [&self.bg2, &self.text2] {
            if !color.is_empty() && !is_hex_color(color) {
                return Err("invalid_hex_color".into());
            }
        }
        if !self.gradient.is_empty() && self.bg2.is_empty() {
            return Err("invalid_gradient".into());
        }
        Ok(())
    }
}

pub fn validate_font_size(size: u32) -> Result<(), String> {
    if size == 0 || (FONT_MIN..=FONT_MAX).contains(&size) {
        Ok(())
    } else {
        Err("invalid_font_size".into())
    }
}

/// `#RRGGBB` (mayusculas o minusculas). Es el formato que guarda toda la app.
pub fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_boton_antiguo_no_cambia_su_json() {
        let style: ButtonStyle = serde_json::from_str("{}").unwrap();
        assert!(style.is_default());
        assert_eq!(serde_json::to_string(&style).unwrap(), "{}");
    }

    #[test]
    fn valida_hex_y_tamanos() {
        assert!(is_hex_color("#3498DB"));
        assert!(is_hex_color("#ff0000"));
        assert!(!is_hex_color("FF0000"));
        assert!(!is_hex_color("#FF00"));
        assert!(!is_hex_color("#GG0000"));
        assert!(validate_font_size(0).is_ok());
        assert!(validate_font_size(18).is_ok());
        assert!(validate_font_size(7).is_err());
        assert!(validate_font_size(49).is_err());
    }

    #[test]
    fn un_degradado_necesita_segundo_color() {
        let mut style = ButtonStyle {
            gradient: "vertical".into(),
            ..Default::default()
        };
        assert!(style.validate().is_err());
        style.bg2 = "#000000".into();
        assert!(style.validate().is_ok());
        style.gradient = "radial".into();
        assert!(style.validate().is_err());
    }
}
