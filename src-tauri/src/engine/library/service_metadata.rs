use super::service::LibraryService;
use super::{embedded_tags, metadata_query, metadata_store};
use crate::domain::library::metadata_fields::{
    LibraryMetadataFields, LibraryMetadataItem, LibraryMetadataResponse,
};

impl LibraryService {
    pub fn metadata_get(&self, paths: &[String]) -> Result<LibraryMetadataResponse, String> {
        validate_selection(paths)?;
        let connection = self.connection()?;
        Ok(LibraryMetadataResponse {
            items: metadata_query::items(&connection, paths)?,
            suggestions: metadata_query::suggestions(&connection, "", 20)?,
        })
    }

    pub fn metadata_save(
        &self,
        paths: &[String],
        fields: &LibraryMetadataFields,
        add_tags: &[String],
        remove_tags: &[String],
        write_to_file: bool,
    ) -> Result<Vec<LibraryMetadataItem>, String> {
        validate_selection(paths)?;
        if write_to_file && paths.len() != 1 {
            return Err("library_metadata_write_unsupported".into());
        }
        let _guard = self
            .operation
            .lock()
            .map_err(|_| "library_operation_lock")?;
        let pending = if write_to_file {
            embedded_tags::prepare(&paths[0], fields)?
        } else {
            None
        };
        if let Some(write) = &pending {
            if let Err(error) = write.install() {
                if let Some(write) = pending {
                    let _ = write.rollback();
                }
                return Err(error);
            }
        }
        let mut connection = self.connection()?;
        if let Err(error) =
            metadata_store::save(&mut connection, paths, fields, add_tags, remove_tags)
        {
            if let Some(write) = pending {
                let _ = write.rollback();
            }
            return Err(public_save_error(error));
        }
        if let Some(write) = &pending {
            write.mark_committed()?;
        }
        if write_to_file {
            refresh_stamp(&connection, &paths[0]);
        }
        if let Some(write) = pending {
            write.commit()?;
        }
        metadata_query::items(&connection, paths)
    }

    pub fn metadata_suggest_tags(&self, query: &str, limit: usize) -> Result<Vec<String>, String> {
        metadata_query::suggestions(&self.connection()?, query, limit)
    }
}

fn public_save_error(error: String) -> String {
    if error == "library_track_not_found" || error.starts_with("invalid_library_metadata_") {
        error
    } else {
        "library_metadata_save_failed".into()
    }
}

fn refresh_stamp(connection: &rusqlite::Connection, path: &str) {
    use crate::engine::audio::formats::stamp_from_metadata;
    use crate::engine::persist::db;
    use rusqlite::params;
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    let (mtime, size) = stamp_from_metadata(&metadata);
    let _ = connection.execute(
        "UPDATE track SET mtime=?2,size=?3 WHERE path=?1",
        params![db::normalize_key(path), mtime, size],
    );
}

fn validate_selection(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() || paths.len() > 500 {
        return Err("invalid_library_metadata_selection".into());
    }
    Ok(())
}
