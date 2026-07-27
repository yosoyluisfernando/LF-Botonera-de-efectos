use super::search_text;

pub fn score(
    query: &str,
    file_name: &str,
    title: Option<&str>,
    artist: Option<&str>,
    album: Option<&str>,
    genre: Option<&str>,
    search_text_value: &str,
) -> Option<i64> {
    let fields = [
        (file_stem(file_name), 9000i64),
        (title.unwrap_or(""), 8000),
        (artist.unwrap_or(""), 5000),
        (album.unwrap_or(""), 3000),
        (genre.unwrap_or(""), 2000),
    ];
    let mut best = 0;
    for (field, weight) in fields {
        let normalized = search_text::normalize(field);
        if normalized == query {
            best = best.max(100_000 + weight);
        } else if normalized.starts_with(query) {
            best = best.max(80_000 + weight);
        } else if normalized.contains(query) {
            best = best.max(60_000 + weight);
        }
    }
    if search_text_value.contains(query) {
        best = best.max(55_000);
    }
    if best > 0 {
        return Some(best);
    }
    fuzzy_score(query, search_text_value)
}

fn fuzzy_score(query: &str, text: &str) -> Option<i64> {
    let candidates = text.split_whitespace().collect::<Vec<_>>();
    let mut total_distance = 0usize;
    for term in query.split_whitespace() {
        let tolerance = tolerance(term.chars().count());
        if tolerance == 0 {
            return None;
        }
        let distance = candidates
            .iter()
            .filter(|candidate| {
                candidate.chars().count().abs_diff(term.chars().count()) <= tolerance
            })
            .map(|candidate| damerau_distance(term, candidate))
            .min()?;
        if distance > tolerance {
            return None;
        }
        total_distance += distance;
    }
    Some(40_000 - total_distance as i64 * 2_000)
}

fn tolerance(length: usize) -> usize {
    match length {
        0..=2 => 0,
        3..=5 => 1,
        6..=9 => 2,
        _ => 3,
    }
}

fn damerau_distance(left: &str, right: &str) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut before_previous = previous.clone();
    for (i, &left_char) in left.iter().enumerate() {
        let mut current = vec![i + 1; right.len() + 1];
        for (j, &right_char) in right.iter().enumerate() {
            let substitution = previous[j] + usize::from(left_char != right_char);
            current[j + 1] = (current[j] + 1).min(previous[j + 1] + 1).min(substitution);
            if i > 0 && j > 0 && left_char == right[j - 1] && left[i - 1] == right_char {
                current[j + 1] = current[j + 1].min(before_previous[j - 1] + 1);
            }
        }
        before_previous = previous;
        previous = current;
    }
    previous[right.len()]
}

fn file_stem(file_name: &str) -> &str {
    file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_beats_fuzzy_and_typo_still_matches() {
        let exact = score(
            "cancion",
            "Canción.mp3",
            None,
            None,
            None,
            None,
            "cancion mp3",
        );
        let typo = score(
            "cansion",
            "Canción.mp3",
            None,
            None,
            None,
            None,
            "cancion mp3",
        );
        assert!(exact > typo);
        assert!(typo.is_some());
    }

    #[test]
    fn adjacent_transposition_is_one_edit() {
        assert_eq!(damerau_distance("efecto", "efecot"), 1);
    }

    #[test]
    fn short_or_unrelated_terms_do_not_match_at_random() {
        assert_eq!(fuzzy_score("ab", "album"), None);
        assert_eq!(fuzzy_score("sirena", "campana"), None);
    }
}
