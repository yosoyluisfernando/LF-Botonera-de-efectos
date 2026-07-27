use std::collections::BTreeSet;

pub fn trigram(query: &str) -> String {
    let tokens = query
        .split_whitespace()
        .filter(|token| token.chars().count() >= 3)
        .collect::<Vec<_>>();
    if tokens.len() == 1 {
        return token(tokens[0]).unwrap_or_default();
    }
    tokens
        .iter()
        .enumerate()
        .map(|(fuzzy_index, _)| {
            let parts = tokens
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    if index == fuzzy_index {
                        token(value).unwrap_or_default()
                    } else {
                        format!("\"{value}\"")
                    }
                })
                .collect::<Vec<_>>();
            format!("({})", parts.join(" AND "))
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn token(value: &str) -> Option<String> {
    let mut fragments = BTreeSet::new();
    let chars = value.chars().collect::<Vec<_>>();
    for window in chars.windows(3) {
        fragments.insert(window.iter().collect::<String>());
    }
    (!fragments.is_empty()).then(|| {
        format!(
            "({})",
            fragments
                .into_iter()
                .map(|fragment| format!("\"{fragment}\""))
                .collect::<Vec<_>>()
                .join(" OR ")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_words_anchor_the_fuzzy_word() {
        let expression = trigram("cansion eterna");
        assert!(expression.contains("\"eterna\""));
        assert!(expression.contains(" OR "));
        assert!(expression.contains(" AND "));
    }
}
