//! SAF por flujo: nunca entrega content:// al decodificador ni carga audio en JS.
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_fs::{FsExt, OpenOptions};
#[path = "files_import.rs"]
mod imports;
#[path = "files_android_locutions.rs"]
mod locutions;
pub use locutions::{locution_folder_plugin, pick_locution_folder};
use imports::{import, safe_name, stamp};
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
static EXPORTS: OnceLock<Mutex<HashMap<PathBuf, FilePath>>> = OnceLock::new();
pub fn initialize(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
fn app() -> &'static tauri::AppHandle {
    APP.get().expect("file adapter initialized")
}
fn exports() -> &'static Mutex<HashMap<PathBuf, FilePath>> {
    EXPORTS.get_or_init(|| Mutex::new(HashMap::new()))
}
pub struct FileDialog {
    filters: Vec<(String, Vec<String>)>,
    name: String,
}
impl FileDialog {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            name: String::new(),
        }
    }
    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.filters.push((
            name.into(),
            extensions.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }
    pub fn set_file_name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }
    fn builder(&self) -> tauri_plugin_dialog::FileDialogBuilder<tauri::Wry> {
        let mut b = app().dialog().file();
        for (label, extensions) in &self.filters {
            let refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
            b = b.add_filter(label, &refs);
        }
        if !self.name.is_empty() {
            b = b.set_file_name(&self.name);
        }
        b
    }
    pub fn pick_file(self) -> Result<Option<PathBuf>, String> {
        let Some(uri) = self.builder().blocking_pick_file() else {
            return Ok(None);
        };
        let audio = self.filters.iter().any(|(_, exts)| {
            exts.iter()
                .any(|ext| super::super::audio::formats::is_audio_extension(ext))
        });
        import(uri, audio).map(Some)
    }
    pub fn pick_folder(self) -> Result<Option<PathBuf>, String> {
        // Los proveedores SAF no son directorios observables mediante notify.
        Err("mobile_folder_unavailable".into())
    }
    pub fn save_file(self) -> Result<Option<PathBuf>, String> {
        let Some(uri) = self.builder().blocking_save_file() else {
            return Ok(None);
        };
        let dir = crate::core::platform::data_dir().join("exports");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{}-{}", stamp(), safe_name(&self.name)));
        exports().lock().unwrap().insert(path.clone(), uri);
        Ok(Some(path))
    }
}
pub fn commit_export(path: &Path) -> Result<(), String> {
    let uri = exports()
        .lock()
        .unwrap()
        .remove(path)
        .ok_or("mobile_export_unavailable")?;
    let mut options = OpenOptions::new();
    options.read(false).write(true).truncate(true);
    let mut source = fs::File::open(path).map_err(|e| e.to_string())?;
    let expected = source.metadata().map_err(|e| e.to_string())?.len();
    let mut destination = app().fs().open(uri, options).map_err(|e| e.to_string())?;
    let written = std::io::copy(&mut source, &mut destination).map_err(|e| e.to_string())?;
    destination.flush().map_err(|e| e.to_string())?;
    drop(destination);
    if written != expected {
        return Err("mobile_export_unavailable".into());
    }
    fs::remove_file(path).map_err(|e| e.to_string())
}
