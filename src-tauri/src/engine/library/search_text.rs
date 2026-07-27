use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

pub fn normalize(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut separator = true;
    for character in value.nfd() {
        if is_combining_mark(character) {
            continue;
        }
        if character.is_alphanumeric() {
            for lower in character.to_lowercase() {
                result.push(lower);
            }
            separator = false;
        } else if !separator {
            result.push(' ');
            separator = true;
        }
    }
    if separator {
        result.pop();
    }
    result
}

pub fn build<'a>(fields: impl IntoIterator<Item = Option<&'a str>>) -> String {
    normalize(
        &fields
            .into_iter()
            .flatten()
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_and_punctuation_are_search_equivalent() {
        assert_eq!(normalize("Canción_RÁPIDA.mp3"), "cancion rapida mp3");
    }

    #[test]
    fn build_skips_missing_fields() {
        assert_eq!(build([Some("Tema"), None, Some("Artista")]), "tema artista");
    }
}
