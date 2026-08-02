use super::data::{basic_catalog, catalog};
use super::types::{PreparedEntry, VisualCatalogItem, VisualCatalogPage, VisualGroup};
use crate::domain::search_text;

// La cuadrícula virtual conserva como máximo 300 elementos. El límite evita
// respuestas arbitrariamente grandes sin obligar a repetir tres búsquedas.
const MAX_PAGE: usize = 300;

pub fn search(
    language: &str,
    query: &str,
    group: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<VisualCatalogPage, String> {
    search_entries(catalog(language)?, query, group, offset, limit, true)
}

pub fn search_visible(
    language: &str,
    query: &str,
    group: Option<&str>,
    offset: u32,
    limit: u32,
    include_skin_tones: bool,
) -> Result<VisualCatalogPage, String> {
    search_entries(
        catalog(language)?,
        query,
        group,
        offset,
        limit,
        include_skin_tones,
    )
}

pub fn search_basics(
    language: &str,
    query: &str,
    group: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<VisualCatalogPage, String> {
    search_entries(basic_catalog(language)?, query, group, offset, limit, true)
}

fn search_entries(
    entries: &[PreparedEntry],
    query: &str,
    group: Option<&str>,
    offset: u32,
    limit: u32,
    include_skin_tones: bool,
) -> Result<VisualCatalogPage, String> {
    if limit == 0 || limit as usize > MAX_PAGE {
        return Err("invalid_page_size".to_string());
    }
    let query = search_text::normalize(query);
    let mut matches = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| include_skin_tones || !has_skin_tone(&entry.value))
        .filter(|(_, entry)| group.is_none_or(|id| entry.group == id))
        .filter_map(|(position, entry)| score(entry, &query).map(|score| (score, position, entry)))
        .collect::<Vec<_>>();
    if !query.is_empty() {
        matches.sort_by_key(|(score, position, _)| (std::cmp::Reverse(*score), *position));
    }
    let total = matches.len();
    let offset = (offset as usize).min(total);
    let end = (offset + limit as usize).min(total);
    let items = matches[offset..end]
        .iter()
        .map(|(_, _, entry)| view(entry))
        .collect();
    Ok(VisualCatalogPage {
        items,
        total: total as u32,
        offset: offset as u32,
        has_more: end < total,
    })
}

pub fn groups(language: &str) -> Result<Vec<VisualGroup>, String> {
    group_counts(catalog(language)?, true)
}

pub fn groups_visible(
    language: &str,
    include_skin_tones: bool,
) -> Result<Vec<VisualGroup>, String> {
    group_counts(catalog(language)?, include_skin_tones)
}

pub fn basic_groups(language: &str) -> Result<Vec<VisualGroup>, String> {
    group_counts(basic_catalog(language)?, true)
}

fn group_counts(
    entries: &[PreparedEntry],
    include_skin_tones: bool,
) -> Result<Vec<VisualGroup>, String> {
    let mut counts = Vec::<VisualGroup>::new();
    for entry in entries
        .iter()
        .filter(|entry| include_skin_tones || !has_skin_tone(&entry.value))
    {
        if let Some(group) = counts.iter_mut().find(|group| group.id == entry.group) {
            group.count += 1;
        } else {
            counts.push(VisualGroup {
                id: entry.group.clone(),
                count: 1,
            });
        }
    }
    Ok(counts)
}

fn has_skin_tone(value: &str) -> bool {
    value
        .chars()
        .any(|character| ('\u{1F3FB}'..='\u{1F3FF}').contains(&character))
}

fn score(entry: &PreparedEntry, query: &str) -> Option<u8> {
    if query.is_empty() {
        return Some(1);
    }
    if entry.normalized_name == query {
        return Some(100);
    }
    if entry.normalized_keywords.iter().any(|value| value == query) {
        return Some(90);
    }
    if entry.normalized_name.starts_with(query) {
        return Some(80);
    }
    if entry
        .normalized_keywords
        .iter()
        .any(|value| value.starts_with(query))
    {
        return Some(70);
    }
    if entry.normalized_name.contains(query) {
        return Some(60);
    }
    if entry.search_text.contains(query) {
        return Some(50);
    }
    query
        .split_whitespace()
        .all(|term| {
            entry
                .search_text
                .split_whitespace()
                .any(|word| word.starts_with(term))
        })
        .then_some(40)
}

fn view(entry: &PreparedEntry) -> VisualCatalogItem {
    VisualCatalogItem {
        value: entry.value.clone(),
        group: entry.group.clone(),
        subgroup: entry.subgroup.clone(),
        name: entry.name.clone(),
    }
}
