//! Construcción única de un botón de audio desde una ruta validada.
use super::defaults::new_button_preserving_colors;
use crate::engine::audio::formats::{probe_duration_secs, validate_audio_file};
use crate::model::ButtonData;

pub fn from_audio_file(
    prefix: &str,
    index: u32,
    path: String,
    theme: &str,
) -> Result<ButtonData, String> {
    from_audio_file_preserving(prefix, index, path, theme, None)
}

pub fn from_audio_file_preserving(
    prefix: &str,
    index: u32,
    path: String,
    theme: &str,
    previous: Option<&ButtonData>,
) -> Result<ButtonData, String> {
    validate_audio_file(&path)?;
    let name = std::path::Path::new(&path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    let mut button = new_button_preserving_colors(prefix, index, &name, theme, previous);
    button.duration = probe_duration_secs(&path);
    button.duration_str = if button.duration > 0.0 {
        format!("{:.1}s", button.duration)
    } else {
        String::new()
    };
    button.path = path;
    Ok(button)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacing_audio_changes_media_but_keeps_the_existing_style() {
        let path = std::env::temp_dir().join(format!("lf-replacement-{}.wav", std::process::id()));
        let mut wav = Vec::new();
        wav.extend(b"RIFF");
        wav.extend(996_u32.to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16_u32.to_le_bytes());
        wav.extend(1_u16.to_le_bytes());
        wav.extend(1_u16.to_le_bytes());
        wav.extend(48000_u32.to_le_bytes());
        wav.extend(96000_u32.to_le_bytes());
        wav.extend(2_u16.to_le_bytes());
        wav.extend(16_u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend(960_u32.to_le_bytes());
        wav.resize(1004, 0);
        std::fs::write(&path, wav).unwrap();
        let mut old = super::super::defaults::new_button("p", 1, "Antes", "#598042", "#FFD700");
        old.path = "old.mp3".into();
        old.duration = 99.0;
        old.style = crate::model::ButtonStyle {
            gradient: "vertical".into(),
            bg2: "#021927".into(),
            text2: "#FFFFFF".into(),
            exact: true,
        };
        let result = from_audio_file_preserving(
            "p",
            1,
            path.to_string_lossy().into_owned(),
            "light",
            Some(&old),
        );
        std::fs::remove_file(path).unwrap();
        let new = result.unwrap();
        assert_eq!(new.color_bg, old.color_bg);
        assert_eq!(new.color_text, old.color_text);
        assert_eq!(new.style, old.style);
        assert_ne!(new.path, old.path);
        assert!((new.duration - 0.01).abs() < 0.001);
    }
}
