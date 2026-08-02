use super::super::*;
use super::first;
use std::collections::HashSet;

#[test]
fn every_language_loads_the_complete_emoji_catalog() {
    for language in ["es", "en", "pt-BR", "pt-PT"] {
        let page = search(language, "", None, 0, 1).unwrap();
        assert_eq!(page.total, 3953);
        assert_eq!(page.items.len(), 1);
        assert!(page.has_more);
    }
}

#[test]
fn spanish_emoji_search_ignores_accents_and_uses_keywords() {
    assert_eq!(first("es", "futbol"), "⚽");
    assert_eq!(first("es", "aplausos"), "👏");
    assert_eq!(first("es", "telefono"), "☎️");
}

#[test]
fn category_terms_find_the_complete_concept_and_skin_tones_can_be_hidden() {
    let singular = search_visible("es", "animal", None, 0, 300, false).unwrap();
    let plural = search_visible("es", "animales", None, 0, 300, false).unwrap();
    assert_eq!(singular.total, 160);
    assert_eq!(plural.total, 160);
    assert!(singular
        .items
        .iter()
        .all(|item| item.group == "animals-and-nature"));
    assert_eq!(
        search_visible("es", "", None, 0, 1, false).unwrap().total,
        1918
    );
    assert_eq!(
        groups_visible("es", false)
            .unwrap()
            .iter()
            .map(|group| group.count)
            .sum::<u32>(),
        1918
    );
}

#[test]
fn portuguese_emoji_catalogs_use_their_own_names() {
    for language in ["pt-BR", "pt-PT"] {
        assert!(search(language, "futebol", None, 0, 10)
            .unwrap()
            .items
            .iter()
            .any(|item| item.value == "⚽"));
    }
}

#[test]
fn group_filter_and_page_bounds_are_enforced() {
    let page = search("es", "", Some("activities"), 0, 10).unwrap();
    assert_eq!(page.items.len(), 10);
    assert!(page.items.iter().all(|item| item.group == "activities"));
    assert_eq!(
        search("es", "", None, 0, 0).unwrap_err(),
        "invalid_page_size"
    );
    assert_eq!(
        search("es", "", None, 0, 301).unwrap_err(),
        "invalid_page_size"
    );
    assert_eq!(search("es", "", None, 0, 300).unwrap().items.len(), 300);
}

#[test]
fn unknown_language_is_rejected() {
    assert_eq!(
        search("fr", "musique", None, 0, 10).unwrap_err(),
        "invalid_language"
    );
}

#[test]
fn emoji_values_are_unique_and_named() {
    for language in ["es", "en", "pt-BR", "pt-PT"] {
        let entries = super::super::data::catalog(language).unwrap();
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.value.as_str())
                .collect::<HashSet<_>>()
                .len(),
            3953
        );
        assert!(entries.iter().all(|entry| !entry.name.trim().is_empty()));
    }
}

#[test]
fn emoji_groups_cover_the_complete_catalog() {
    assert_eq!(
        groups("es")
            .unwrap()
            .into_iter()
            .map(|group| group.count)
            .sum::<u32>(),
        3953
    );
}

#[test]
fn expanded_catalogs_are_localized_searchable_and_grouped() {
    for language in ["es", "en", "pt-BR", "pt-PT"] {
        assert!(search_basics(language, "", None, 0, 1).unwrap().total > 8000);
    }
    let dogs = search_basics("es", "perro", None, 0, 300).unwrap();
    assert!(dogs
        .items
        .iter()
        .any(|item| item.value == "tabler:animals:dog"));
    let animals = search_basics("es", "animales", Some("animals"), 0, 300).unwrap();
    assert!(animals.total > 100);
    assert!(animals.items.iter().all(|item| item.group == "animals"));
}

#[test]
fn expanded_group_totals_match_their_catalogs() {
    let basics = search_basics("es", "", None, 0, 1).unwrap().total;
    assert_eq!(
        basic_groups("es")
            .unwrap()
            .iter()
            .map(|group| group.count)
            .sum::<u32>(),
        basics
    );
}
