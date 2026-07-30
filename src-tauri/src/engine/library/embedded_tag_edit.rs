//! Edición y verificación de etiquetas sobre una copia temporal.
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag};
use std::fs::OpenOptions;
use std::path::Path;

pub(super) fn edit_and_verify(
    path: &Path,
    values: &[(&'static str, String)],
) -> Result<(), String> {
    let mut file = Probe::open(path)
        .and_then(|probe| probe.read())
        .map_err(|_| "library_metadata_write_unsupported")?;
    let duration = file.properties().duration();
    let tag_type = file.primary_tag_type();
    if file.primary_tag_mut().is_none() {
        file.insert_tag(Tag::new(tag_type));
    }
    let tag = file
        .primary_tag_mut()
        .ok_or("library_metadata_write_unsupported")?;
    for (field, value) in values {
        apply(tag, field, value)?;
    }
    file.save_to_path(path, WriteOptions::default())
        .map_err(|_| "library_metadata_write_failed")?;
    sync(path)?;
    let verified = Probe::open(path)
        .and_then(|probe| probe.read())
        .map_err(|_| "library_metadata_write_verify_failed")?;
    if verified.properties().duration() != duration {
        return Err("library_metadata_write_verify_failed".into());
    }
    let tag = verified
        .primary_tag()
        .or_else(|| verified.first_tag())
        .ok_or("library_metadata_write_verify_failed")?;
    for (field, value) in values {
        verify(tag, field, value)?;
    }
    Ok(())
}

pub(super) fn sync(path: &Path) -> Result<(), String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| "library_metadata_write_failed".into())
}

fn apply(tag: &mut Tag, field: &str, value: &str) -> Result<(), String> {
    match field {
        "title" if value.is_empty() => tag.remove_title(),
        "title" => tag.set_title(value.to_string()),
        "artist" if value.is_empty() => tag.remove_artist(),
        "artist" => tag.set_artist(value.to_string()),
        "album" if value.is_empty() => tag.remove_album(),
        "album" => tag.set_album(value.to_string()),
        "genre" if value.is_empty() => tag.remove_genre(),
        "genre" => tag.set_genre(value.to_string()),
        "comment" if value.is_empty() => tag.remove_comment(),
        "comment" => tag.set_comment(value.to_string()),
        "year" if value.is_empty() => tag.remove_year(),
        "year" => tag.set_year(value.parse().map_err(|_| "library_metadata_write_failed")?),
        "track_number" if value.is_empty() => tag.remove_track(),
        "track_number" => {
            tag.set_track(value.parse().map_err(|_| "library_metadata_write_failed")?)
        }
        "album_artist" => item(tag, ItemKey::AlbumArtist, value)?,
        "composer" => item(tag, ItemKey::Composer, value)?,
        _ => return Ok(()),
    }
    Ok(())
}

fn item(tag: &mut Tag, key: ItemKey, value: &str) -> Result<(), String> {
    tag.remove_key(&key);
    if !value.is_empty() && !tag.insert_text(key, value.to_string()) {
        return Err("library_metadata_write_unsupported".into());
    }
    Ok(())
}

fn verify(tag: &Tag, field: &str, value: &str) -> Result<(), String> {
    let actual = match field {
        "title" => tag.title().map(|value| value.into_owned()),
        "artist" => tag.artist().map(|value| value.into_owned()),
        "album" => tag.album().map(|value| value.into_owned()),
        "genre" => tag.genre().map(|value| value.into_owned()),
        "comment" => tag.comment().map(|value| value.into_owned()),
        "year" => tag.year().map(|value| value.to_string()),
        "track_number" => tag.track().map(|value| value.to_string()),
        "album_artist" => tag.get_string(&ItemKey::AlbumArtist).map(str::to_string),
        "composer" => tag.get_string(&ItemKey::Composer).map(str::to_string),
        _ => return Ok(()),
    };
    let expected = (!value.is_empty()).then_some(value);
    (actual.as_deref() == expected)
        .then_some(())
        .ok_or_else(|| "library_metadata_write_verify_failed".into())
}
