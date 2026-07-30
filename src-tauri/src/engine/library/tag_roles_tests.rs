use super::*;
use crate::engine::library::search::SearchResult;

#[test]
fn repeated_title_with_varying_artists_is_swapped_within_its_folder() {
    let mut rows = vec![
        candidate("album/01.mp3", "Los Diablitos", "Historia"),
        candidate("album/02.mp3", "Los Diablitos", "A Besitos"),
        candidate("album/03.mp3", "Los Diablitos", "Novios Cruzados"),
    ];

    correct(&mut rows);

    assert_eq!(rows[0].result.title.as_deref(), Some("Historia"));
    assert_eq!(rows[0].result.artist.as_deref(), Some("Los Diablitos"));
}

#[test]
fn correctly_tagged_album_is_not_changed() {
    let mut rows = vec![
        candidate("album/01.mp3", "Historia", "Los Diablitos"),
        candidate("album/02.mp3", "A Besitos", "Los Diablitos"),
        candidate("album/03.mp3", "Novios Cruzados", "Los Diablitos"),
    ];

    correct(&mut rows);

    assert_eq!(rows[0].result.title.as_deref(), Some("Historia"));
    assert_eq!(rows[0].result.artist.as_deref(), Some("Los Diablitos"));
}

#[test]
fn structured_filename_corrects_a_single_inverted_result() {
    let mut rows = vec![candidate(
        "album/002_Historia---Los Diablitos..mp3",
        "Los Diablitos",
        "Historia",
    )];

    correct(&mut rows);

    assert_eq!(rows[0].result.title.as_deref(), Some("Historia"));
    assert_eq!(rows[0].result.artist.as_deref(), Some("Los Diablitos"));
}

fn candidate(path: &str, title: &str, artist: &str) -> Candidate {
    Candidate {
        root_path: "D:/Music".into(),
        relative_path: path.into(),
        result: SearchResult {
            path: String::new(),
            path_key: path.into(),
            collection: "music".into(),
            file_name: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            title: Some(title.into()),
            artist: Some(artist.into()),
            album: None,
            genre: None,
            year: None,
            track_number: None,
            duration_s: 1.0,
            metadata_state: "ready".into(),
        },
        search_text: String::new(),
        user_roles: false,
    }
}
