//! Un solo adaptador de diálogos. Android importa documentos a archivos privados.
#[cfg(target_os = "android")]
#[path = "files_android.rs"]
mod android;
#[cfg(target_os = "android")]
pub use android::{commit_export, initialize, FileDialog};

#[cfg(not(target_os = "android"))]
pub fn initialize(_: tauri::AppHandle) {}
#[cfg(not(target_os = "android"))]
pub fn commit_export(_: &std::path::Path) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub struct FileDialog(rfd::FileDialog);
#[cfg(not(target_os = "android"))]
impl FileDialog {
    pub fn new() -> Self {
        Self(rfd::FileDialog::new())
    }
    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.0 = self.0.add_filter(name, extensions);
        self
    }
    pub fn set_file_name(mut self, name: &str) -> Self {
        self.0 = self.0.set_file_name(name);
        self
    }
    pub fn pick_file(self) -> Result<Option<std::path::PathBuf>, String> {
        Ok(self.0.pick_file())
    }
    pub fn pick_folder(self) -> Result<Option<std::path::PathBuf>, String> {
        Ok(self.0.pick_folder())
    }
    pub fn save_file(self) -> Result<Option<std::path::PathBuf>, String> {
        Ok(self.0.save_file())
    }
}

/// Selecci?n de un audio para la cola; corre fuera del hilo de UI.
#[tauri::command(async)]
pub fn pick_audio_file() -> Result<Option<String>, String> {
    FileDialog::new()
        .add_filter("Audio", crate::engine::audio::formats::AUDIO_EXTENSIONS)
        .pick_file()
        .map(|path| path.map(|p| p.to_string_lossy().into_owned()))
}
