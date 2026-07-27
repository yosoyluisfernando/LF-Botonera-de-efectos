use lofty::config::ParseOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::borrow::Cow;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LibraryMetadata {
    pub duration_s: f64,
    pub sample_rate: u32,
    pub channels: u8,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub tags_readable: bool,
}

/// Lee propiedades y etiquetas en una sola apertura. Si una etiqueta antigua
/// esta mal codificada, conserva al menos las propiedades tecnicas.
pub fn read(path: &str) -> Result<LibraryMetadata, String> {
    match Probe::open(path).and_then(|probe| probe.read()) {
        Ok(file) => {
            let properties = file.properties();
            let tag = file.primary_tag().or_else(|| file.first_tag());
            Ok(LibraryMetadata {
                duration_s: properties.duration().as_secs_f64(),
                sample_rate: properties.sample_rate().unwrap_or(0),
                channels: properties.channels().unwrap_or(0),
                title: clean_text(tag.and_then(|value| value.title())),
                artist: clean_text(tag.and_then(|value| value.artist())),
                album: clean_text(tag.and_then(|value| value.album())),
                genre: clean_text(tag.and_then(|value| value.genre())),
                year: tag.and_then(|value| value.year()),
                track: tag.and_then(|value| value.track()),
                tags_readable: true,
            })
        }
        Err(tag_error) => read_properties_only(path)
            .map_err(|property_error| format!("tags: {tag_error}; properties: {property_error}")),
    }
}

fn clean_text(value: Option<Cow<'_, str>>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn read_properties_only(path: &str) -> Result<LibraryMetadata, lofty::error::LoftyError> {
    let file = Probe::open(path)?
        .options(ParseOptions::new().read_tags(false))
        .read()?;
    let properties = file.properties();
    Ok(LibraryMetadata {
        duration_s: properties.duration().as_secs_f64(),
        sample_rate: properties.sample_rate().unwrap_or(0),
        channels: properties.channels().unwrap_or(0),
        tags_readable: false,
        ..LibraryMetadata::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_tags_do_not_pollute_the_search_index() {
        assert_eq!(
            clean_text(Some(Cow::Borrowed("  Titulo  "))).as_deref(),
            Some("Titulo")
        );
        assert_eq!(clean_text(Some(Cow::Borrowed("   "))), None);
        assert_eq!(clean_text(None), None);
    }
}
