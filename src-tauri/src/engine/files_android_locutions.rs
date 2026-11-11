//! Android SAF folder import for locution packs.
use super::{app, safe_name, stamp};
use crate::engine::audio::formats::is_audio_path;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::plugin::{Builder, PluginHandle, TauriPlugin};
use tauri::{Manager, Wry};
use tauri_plugin_dialog::FilePath;
use tauri_plugin_fs::{FsExt, OpenOptions};

const IMPORT_ERROR: &str = "locution_folder_import";

struct FolderPlugin(PluginHandle<Wry>);

pub fn locution_folder_plugin() -> TauriPlugin<Wry> {
    Builder::new("android-locutions")
        .setup(|app, api| {
            let native = api.register_android_plugin("lf.botonera.locutions", "FolderPlugin")?;
            app.manage(FolderPlugin(native));
            Ok(())
        })
        .build()
}

#[derive(Deserialize)]
struct PickerResult {
    folder: Option<Folder>,
}

#[derive(Deserialize)]
struct Folder {
    name: String,
    files: Vec<Document>,
}

#[derive(Deserialize)]
struct Document {
    name: String,
    uri: String,
}

pub(super) fn document_name(uri: &FilePath) -> Result<String, String> {
    #[derive(Deserialize)]
    struct NameResult { name: String }
    match uri {
        FilePath::Path(path) => Ok(path.file_name().unwrap_or_default().to_string_lossy().into_owned()),
        FilePath::Url(uri) => {
            let result: NameResult = app().state::<FolderPlugin>().0
                .run_mobile_plugin("fileName", serde_json::json!({ "uri": uri.as_str() }))
                .map_err(|_| "mobile_file_name_unavailable")?;
            Ok(result.name)
        }
    }
}

pub fn pick_locution_folder() -> Result<Option<PathBuf>, String> {
    let response: PickerResult = app()
        .state::<FolderPlugin>()
        .0
        .run_mobile_plugin("pick", ())
        .map_err(|error| {
            eprintln!("Android locution folder picker: {error}");
            IMPORT_ERROR
        })?;
    response.folder.map(import_folder).transpose()
}

fn import_folder(mut folder: Folder) -> Result<PathBuf, String> {
    folder
        .files
        .retain(|file| is_audio_path(Path::new(&file.name)));
    if folder.files.is_empty() {
        return Err("locution_folder_empty".into());
    }

    let root = crate::core::platform::data_dir().join("locutions");
    fs::create_dir_all(&root).map_err(|error| log_import_error(&error))?;
    let batch = root.join(stamp().to_string());
    fs::create_dir(&batch).map_err(|error| log_import_error(&error))?;
    let destination = batch.join(safe_name(&folder.name));
    let result = fs::create_dir(&destination)
        .map_err(|error| log_import_error(&error))
        .and_then(|()| {
            for file in folder.files {
                copy_document(file, &destination).map_err(|error| log_import_error(&error))?;
            }
            Ok(destination)
        });
    if result.is_err() {
        let _ = fs::remove_dir_all(batch);
    }
    result
}

fn copy_document(file: Document, folder: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // The playback resolver looks up the original clip names verbatim.
    if file.name.contains(['/', '\\', '\0']) || !file.uri.starts_with("content://") {
        return Err("Invalid document name or URI".into());
    }
    let uri = FilePath::Url(file.uri.parse()?);
    let mut options = OpenOptions::new();
    options.read(true);
    let mut source = app().fs().open(uri, options)?;
    let mut destination = fs::File::create_new(folder.join(file.name))?;
    std::io::copy(&mut source, &mut destination)?;
    destination.sync_all()?;
    Ok(())
}

fn log_import_error(error: &dyn std::fmt::Display) -> String {
    eprintln!("Android locution folder copy: {error}");
    IMPORT_ERROR.into()
}
