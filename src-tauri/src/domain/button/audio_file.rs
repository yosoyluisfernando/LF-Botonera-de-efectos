//! Construcción única de un botón de audio desde una ruta validada.
use super::defaults::new_button;
use crate::engine::audio::formats::{probe_duration_secs, validate_audio_file};
use crate::model::ButtonData;

pub fn from_audio_file(
    prefix: &str,
    index: u32,
    path: String,
    theme: &str,
) -> Result<ButtonData, String> {
    validate_audio_file(&path)?;
    let name = std::path::Path::new(&path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    let bg = crate::domain::colors::random_color();
    let text = crate::domain::colors::text_for_theme(&bg, theme, "button");
    let mut button = new_button(prefix, index, &name, &bg, &text);
    button.duration = probe_duration_secs(&path);
    button.duration_str = if button.duration > 0.0 {
        format!("{:.1}s", button.duration)
    } else {
        String::new()
    };
    button.path = path;
    Ok(button)
}
