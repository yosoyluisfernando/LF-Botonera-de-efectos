/// Modulo: colors.rs
/// Proposito: paleta segura, contraste y colores para botones nuevos.
use serde::Serialize;

const SAFE_COLORS: [&str; 32] = [
    "#E53935", "#D81B60", "#8E24AA", "#5E35B1", "#3949AB", "#1E88E5", "#039BE5", "#00ACC1",
    "#00897B", "#43A047", "#7CB342", "#C0CA33", "#FDD835", "#FFB300", "#FB8C00", "#F4511E",
    "#6D4C41", "#546E7A", "#C62828", "#AD1457", "#6A1B9A", "#4527A0", "#283593", "#1565C0",
    "#0277BD", "#00838F", "#00695C", "#2E7D32", "#558B2F", "#9E9D24", "#EF6C00", "#D84315",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorOption {
    pub base: &'static str,
    pub dark_bg: String,
    pub dark_text: &'static str,
    pub light_bg: String,
    pub light_text: String,
}

/// Devuelve la paleta que la UI debe ofrecer al usuario.
pub fn color_palette() -> Vec<ColorOption> {
    SAFE_COLORS
        .iter()
        .map(|c| {
            let light_bg = adapt_color(c, "light", "button");
            ColorOption {
                base: c,
                dark_bg: adapt_color(c, "dark", "button"),
                dark_text: "#FFFFFF",
                light_text: readable_text(&light_bg),
                light_bg,
            }
        })
        .collect()
}

/// Color pseudoaleatorio elegido desde una paleta curada.
pub(crate) fn random_color() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    SAFE_COLORS[((nanos ^ n.wrapping_mul(137)) as usize) % SAFE_COLORS.len()].to_string()
}

pub fn text_for_theme(base: &str, theme: &str, role: &str) -> String {
    if theme == "dark" && matches!(role, "button" | "tab") {
        return "#FFFFFF".to_string();
    }
    readable_text(&adapt_color(base, theme, role))
}

fn adapt_color(hex: &str, theme: &str, role: &str) -> String {
    let Some((h, mut s, mut l)) = hex_to_hsl(hex) else {
        return hex.to_string();
    };
    if theme == "light" {
        s = (s * 1.35)
            .max(if role == "tab" { 0.82 } else { 0.9 })
            .min(1.0);
        l = l.max(0.5).min(0.68);
    } else {
        s = (s * if role == "tab" { 0.92 } else { 0.82 }).clamp(0.28, 0.66);
        l = l
            .min(if role == "tab" { 0.34 } else { 0.3 })
            .clamp(0.16, 0.34);
    }
    hsl_to_hex(h, s, l)
}

fn hsl_to_hex(h: f32, s: f32, l: f32) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    format!(
        "#{:02X}{:02X}{:02X}",
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8
    )
}

fn hex_to_hsl(hex: &str) -> Option<(f32, f32, f32)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).ok()? as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min {
        return Some((0.0, 0.0, l));
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    Some((h * 360.0, s, l))
}

fn readable_text(bg_hex: &str) -> String {
    let Some((r, g, b)) = hex_to_rgb(bg_hex) else {
        return "#FFFFFF".to_string();
    };
    let luminance = 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
    if luminance > 0.45 {
        "#111111"
    } else {
        "#FFFFFF"
    }
    .to_string()
}

fn hex_to_rgb(hex: &str) -> Option<(f32, f32, f32)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&h[0..2], 16).ok()? as f32 / 255.0,
        u8::from_str_radix(&h[2..4], 16).ok()? as f32 / 255.0,
        u8::from_str_radix(&h[4..6], 16).ok()? as f32 / 255.0,
    ))
}

fn channel(v: f32) -> f32 {
    if v <= 0.03928 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
