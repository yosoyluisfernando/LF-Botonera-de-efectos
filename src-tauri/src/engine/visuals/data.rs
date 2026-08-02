use super::types::{CatalogEntry, CatalogHeader, PreparedEntry};
use crate::domain::search_text;
use std::sync::OnceLock;

const ES: &str = include_str!("../../../resources/emoji/es.ndjson");
const EN: &str = include_str!("../../../resources/emoji/en.ndjson");
const PT_BR: &str = include_str!("../../../resources/emoji/pt-BR.ndjson");
const PT_PT: &str = include_str!("../../../resources/emoji/pt-PT.ndjson");
const BASIC_ES: &str = include_str!("../../../resources/basic/es.ndjson");
const BASIC_EN: &str = include_str!("../../../resources/basic/en.ndjson");
const BASIC_PT_BR: &str = include_str!("../../../resources/basic/pt-BR.ndjson");
const BASIC_PT_PT: &str = include_str!("../../../resources/basic/pt-PT.ndjson");

static ES_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static EN_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static PT_BR_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static PT_PT_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static BASIC_ES_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static BASIC_EN_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static BASIC_PT_BR_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();
static BASIC_PT_PT_DATA: OnceLock<Result<Vec<PreparedEntry>, String>> = OnceLock::new();

pub(super) fn catalog(language: &str) -> Result<&'static [PreparedEntry], String> {
    let (cell, source) = match language {
        "es" => (&ES_DATA, ES),
        "en" => (&EN_DATA, EN),
        "pt-BR" => (&PT_BR_DATA, PT_BR),
        "pt-PT" => (&PT_PT_DATA, PT_PT),
        _ => return Err("invalid_language".to_string()),
    };
    match cell.get_or_init(|| parse(source, language, 1)) {
        Ok(entries) => Ok(entries),
        Err(error) => Err(error.clone()),
    }
}

pub(super) fn contains_value(value: &str) -> Result<bool, String> {
    Ok(catalog("en")?.iter().any(|entry| entry.value == value))
}

pub(super) fn basic_catalog(language: &str) -> Result<&'static [PreparedEntry], String> {
    let (cell, source) = match language {
        "es" => (&BASIC_ES_DATA, BASIC_ES),
        "en" => (&BASIC_EN_DATA, BASIC_EN),
        "pt-BR" => (&BASIC_PT_BR_DATA, BASIC_PT_BR),
        "pt-PT" => (&BASIC_PT_PT_DATA, BASIC_PT_PT),
        _ => return Err("invalid_language".to_string()),
    };
    match cell.get_or_init(|| parse(source, language, 2)) {
        Ok(entries) => Ok(entries),
        Err(error) => Err(error.clone()),
    }
}

pub(super) fn contains_basic_value(value: &str) -> Result<bool, String> {
    Ok(basic_catalog("en")?
        .iter()
        .any(|entry| entry.value == value))
}

fn parse(
    source: &str,
    expected_language: &str,
    expected_format: u32,
) -> Result<Vec<PreparedEntry>, String> {
    let mut lines = source.lines();
    let header: CatalogHeader = serde_json::from_str(
        lines
            .next()
            .ok_or_else(|| "emoji_catalog_empty".to_string())?,
    )
    .map_err(|error| format!("emoji_catalog_header: {error}"))?;
    if header.format != expected_format || header.locale != expected_language {
        return Err("emoji_catalog_version".to_string());
    }
    let entries = lines
        .map(|line| parse_entry(line, expected_language))
        .collect::<Result<Vec<_>, _>>()?;
    if entries.len() != header.count {
        return Err("emoji_catalog_count".to_string());
    }
    Ok(entries)
}

fn parse_entry(line: &str, language: &str) -> Result<PreparedEntry, String> {
    let raw: CatalogEntry =
        serde_json::from_str(line).map_err(|error| format!("emoji_catalog_entry: {error}"))?;
    let normalized_name = search_text::normalize(&raw.name);
    let normalized_keywords = raw
        .keywords
        .iter()
        .map(|keyword| search_text::normalize(keyword))
        .collect::<Vec<_>>();
    let search_text = std::iter::once(normalized_name.as_str())
        .chain(normalized_keywords.iter().map(String::as_str))
        .chain(std::iter::once(super::category_terms::for_group(
            language, &raw.group,
        )))
        .collect::<Vec<_>>()
        .join(" ");
    Ok(PreparedEntry {
        value: raw.value,
        group: raw.group,
        subgroup: raw.subgroup,
        name: raw.name,
        normalized_name,
        normalized_keywords,
        search_text,
    })
}
