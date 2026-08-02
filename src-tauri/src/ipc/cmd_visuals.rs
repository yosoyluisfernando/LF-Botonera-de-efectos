//! Consulta de los catálogos visuales locales.
use crate::engine::visuals::{self, VisualCatalogPage, VisualGroup};

#[tauri::command]
pub fn visuals_search_emojis(
    language: String,
    query: String,
    group: Option<String>,
    offset: u32,
    limit: u32,
    include_skin_tones: Option<bool>,
) -> Result<VisualCatalogPage, String> {
    visuals::search_visible(
        &language,
        &query,
        group.as_deref(),
        offset,
        limit,
        include_skin_tones.unwrap_or(false),
    )
}

#[tauri::command]
pub fn visuals_emoji_groups(
    language: String,
    include_skin_tones: Option<bool>,
) -> Result<Vec<VisualGroup>, String> {
    visuals::groups_visible(&language, include_skin_tones.unwrap_or(false))
}

#[tauri::command]
pub fn visuals_search_basics(
    language: String,
    query: String,
    group: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<VisualCatalogPage, String> {
    visuals::search_basics(&language, &query, group.as_deref(), offset, limit)
}

#[tauri::command]
pub fn visuals_basic_groups(language: String) -> Result<Vec<VisualGroup>, String> {
    visuals::basic_groups(&language)
}
