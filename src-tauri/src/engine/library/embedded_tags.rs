//! Escritura opcional de etiquetas sobre una copia verificada del audio.
use crate::domain::library::metadata_fields::LibraryMetadataFields;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PendingTagWrite {
    journal: super::tag_write_journal::TagWriteJournal,
    data_dir: PathBuf,
}

impl PendingTagWrite {
    pub fn install(&self) -> Result<(), String> {
        super::tag_write_journal::store_at(&self.data_dir, &self.journal)?;
        fs::rename(&self.journal.source, &self.journal.backup)
            .map_err(|_| "library_metadata_write_locked")?;
        if fs::rename(&self.journal.staged, &self.journal.source).is_err() {
            let _ = fs::rename(&self.journal.backup, &self.journal.source);
            return Err("library_metadata_write_failed".into());
        }
        super::embedded_tag_edit::sync(&self.journal.source)
    }

    pub fn commit(self) -> Result<(), String> {
        super::tag_write_journal::remove_if_exists(&self.journal.backup)?;
        super::tag_write_journal::clear_at(&self.data_dir)
    }

    pub fn mark_committed(&self) -> Result<(), String> {
        let journal = super::tag_write_journal::TagWriteJournal {
            committed: true,
            source: self.journal.source.clone(),
            staged: self.journal.staged.clone(),
            backup: self.journal.backup.clone(),
        };
        super::tag_write_journal::store_at(&self.data_dir, &journal)
    }

    pub fn rollback(self) -> Result<(), String> {
        if self.journal.backup.exists() {
            super::tag_write_journal::remove_if_exists(&self.journal.source)?;
            fs::rename(&self.journal.backup, &self.journal.source)
                .map_err(|_| "library_metadata_write_failed")?;
        }
        super::tag_write_journal::remove_if_exists(&self.journal.staged)?;
        super::tag_write_journal::clear_at(&self.data_dir)
    }
}

pub fn prepare(
    path: &str,
    fields: &LibraryMetadataFields,
) -> Result<Option<PendingTagWrite>, String> {
    prepare_at(
        &crate::engine::persist::config_io::get_data_dir(),
        path,
        fields,
    )
}

pub fn prepare_at(
    data_dir: &Path,
    path: &str,
    fields: &LibraryMetadataFields,
) -> Result<Option<PendingTagWrite>, String> {
    let values = fields.values()?;
    if !values.iter().any(|(name, _)| embeddable(name)) {
        return Ok(None);
    }
    let source = PathBuf::from(path);
    let extension = source.extension().and_then(|value| value.to_str());
    if extension.map_or(true, |value| value.eq_ignore_ascii_case("wma")) {
        return Err("library_metadata_write_unsupported".into());
    }
    let staged = sibling(&source, "lf-tags-staged")?;
    let backup = sibling(&source, "lf-tags-backup")?;
    super::tag_write_journal::remove_if_exists(&staged)?;
    super::tag_write_journal::remove_if_exists(&backup)?;
    fs::copy(&source, &staged).map_err(|_| "library_metadata_write_failed")?;
    let result = super::embedded_tag_edit::edit_and_verify(&staged, &values);
    if let Err(error) = result {
        let _ = super::tag_write_journal::remove_if_exists(&staged);
        return Err(error);
    }
    Ok(Some(PendingTagWrite {
        journal: super::tag_write_journal::TagWriteJournal {
            source,
            staged,
            backup,
            committed: false,
        },
        data_dir: data_dir.to_path_buf(),
    }))
}

fn embeddable(field: &str) -> bool {
    matches!(
        field,
        "title"
            | "artist"
            | "album"
            | "album_artist"
            | "genre"
            | "year"
            | "track_number"
            | "composer"
            | "comment"
    )
}

fn sibling(path: &Path, marker: &str) -> Result<PathBuf, String> {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or("library_metadata_write_unsupported")?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    Ok(path.with_file_name(format!(
        ".{stem}.{marker}-{}.{}",
        std::process::id(),
        extension
    )))
}
