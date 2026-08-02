use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct CatalogHeader {
    pub format: u32,
    pub locale: String,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct CatalogEntry {
    pub value: String,
    pub group: String,
    pub subgroup: String,
    pub name: String,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub(super) struct PreparedEntry {
    pub value: String,
    pub group: String,
    pub subgroup: String,
    pub name: String,
    pub normalized_name: String,
    pub normalized_keywords: Vec<String>,
    pub search_text: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct VisualCatalogItem {
    pub value: String,
    pub group: String,
    pub subgroup: String,
    pub name: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VisualCatalogPage {
    pub items: Vec<VisualCatalogItem>,
    pub total: u32,
    pub offset: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VisualGroup {
    pub id: String,
    pub count: u32,
}
