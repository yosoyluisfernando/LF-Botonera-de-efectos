mod catalog;
mod validation;

use super::search;

fn first(language: &str, query: &str) -> String {
    search(language, query, None, 0, 1).unwrap().items[0]
        .value
        .clone()
}
