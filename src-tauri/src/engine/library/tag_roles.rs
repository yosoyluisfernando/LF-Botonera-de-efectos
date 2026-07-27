//! Corrige para presentación lotes cuyos tags título/artista vienen invertidos.
use super::search::Candidate;
use super::search_text;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const MIN_REPEATED_TITLE: usize = 3;

#[derive(Default)]
struct FolderEvidence {
    titles: HashMap<String, usize>,
    artists: HashMap<String, usize>,
    distinct_artists: HashSet<String>,
}

pub fn correct(candidates: &mut [Candidate]) {
    let mut folders = HashMap::<String, FolderEvidence>::new();
    for candidate in candidates.iter() {
        let evidence = folders.entry(parent(&candidate.relative_path)).or_default();
        count(&mut evidence.titles, candidate.result.title.as_deref());
        count(&mut evidence.artists, candidate.result.artist.as_deref());
        if let Some(artist) = normalized(candidate.result.artist.as_deref()) {
            evidence.distinct_artists.insert(artist);
        }
    }
    for candidate in candidates {
        let Some(evidence) = folders.get(&parent(&candidate.relative_path)) else {
            continue;
        };
        let Some(title) = normalized(candidate.result.title.as_deref()) else {
            continue;
        };
        let Some(artist) = normalized(candidate.result.artist.as_deref()) else {
            continue;
        };
        let title_repeats = evidence.titles.get(&title).copied().unwrap_or(0);
        let artist_repeats = evidence.artists.get(&artist).copied().unwrap_or(0);
        let folder_evidence = title_repeats >= MIN_REPEATED_TITLE
            && title_repeats > artist_repeats
            && evidence.distinct_artists.len() >= 2;
        if folder_evidence || structured_file_evidence(candidate, &title, &artist) {
            std::mem::swap(&mut candidate.result.title, &mut candidate.result.artist);
        }
    }
}

fn count(target: &mut HashMap<String, usize>, value: Option<&str>) {
    if let Some(value) = normalized(value) {
        *target.entry(value).or_default() += 1;
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value
        .map(search_text::normalize)
        .filter(|value| !value.is_empty())
}

fn parent(relative_path: &str) -> String {
    Path::new(relative_path)
        .parent()
        .map(|value| value.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn structured_file_evidence(candidate: &Candidate, title: &str, artist: &str) -> bool {
    let stem = Path::new(&candidate.result.file_name)
        .file_stem()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();
    let Some((left, right)) = stem.split_once("--") else {
        return false;
    };
    let left = search_text::normalize(left);
    let right = search_text::normalize(right);
    left.contains(artist) && right.contains(title)
}

#[cfg(test)]
#[path = "tag_roles_tests.rs"]
mod tests;
