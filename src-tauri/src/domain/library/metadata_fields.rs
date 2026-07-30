//! Contratos y validación de metadatos internos de la Biblioteca.
use serde::{Deserialize, Serialize};

pub const FIELD_NAMES: [&str; 12] = [
    "title",
    "artist",
    "album",
    "album_artist",
    "genre",
    "year",
    "track_number",
    "composer",
    "comment",
    "display_name",
    "category",
    "description",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum NumberInput {
    Number(u32),
    Text(String),
}

impl NumberInput {
    pub fn storage(&self, field: &str) -> Result<String, String> {
        let text = match self {
            Self::Number(value) => value.to_string(),
            Self::Text(value) => value.trim().to_string(),
        };
        if text.is_empty() {
            return Ok(text);
        }
        let value = text
            .parse::<u32>()
            .map_err(|_| format!("invalid_library_metadata_{field}"))?;
        let valid = match field {
            "year" => value <= 9999,
            "track_number" => (1..=9999).contains(&value),
            _ => false,
        };
        valid
            .then_some(value.to_string())
            .ok_or_else(|| format!("invalid_library_metadata_{field}"))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LibraryMetadataFields {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    #[serde(alias = "albumArtist")]
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<NumberInput>,
    #[serde(alias = "trackNumber")]
    pub track_number: Option<NumberInput>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    #[serde(alias = "displayName")]
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

impl LibraryMetadataFields {
    pub fn values(&self) -> Result<Vec<(&'static str, String)>, String> {
        let mut values = Vec::new();
        push_text(&mut values, "title", &self.title)?;
        push_text(&mut values, "artist", &self.artist)?;
        push_text(&mut values, "album", &self.album)?;
        push_text(&mut values, "album_artist", &self.album_artist)?;
        push_text(&mut values, "genre", &self.genre)?;
        if let Some(value) = &self.year {
            values.push(("year", value.storage("year")?));
        }
        if let Some(value) = &self.track_number {
            values.push(("track_number", value.storage("track_number")?));
        }
        push_text(&mut values, "composer", &self.composer)?;
        push_text(&mut values, "comment", &self.comment)?;
        push_text(&mut values, "display_name", &self.display_name)?;
        push_text(&mut values, "category", &self.category)?;
        push_text(&mut values, "description", &self.description)?;
        Ok(values)
    }
}

fn push_text(
    target: &mut Vec<(&'static str, String)>,
    field: &'static str,
    value: &Option<String>,
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let value = value.trim();
    if value.chars().count() > 500 {
        return Err(format!("invalid_library_metadata_{field}"));
    }
    target.push((field, value.to_string()));
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LibraryMetadataItem {
    pub path: String,
    pub collection: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryMetadataResponse {
    pub items: Vec<LibraryMetadataItem>,
    pub suggestions: Vec<String>,
}

pub fn clean_tag(value: &str) -> Result<Option<String>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > 80 {
        return Err("invalid_library_metadata_tag".into());
    }
    Ok(Some(value.to_string()))
}
