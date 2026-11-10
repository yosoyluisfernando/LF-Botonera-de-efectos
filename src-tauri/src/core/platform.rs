//! Capacidades reales de la edición; no dependen del navegador ni del tamaño.
use serde::Serialize;
#[cfg(target_os = "android")]
use std::{path::PathBuf, sync::OnceLock};
#[cfg(target_os = "android")]
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

#[cfg(target_os = "android")]
#[path = "platform_android_audio.rs"]
mod android_audio;

#[derive(Serialize)]
pub struct Capabilities {
    pub platform: &'static str,
    pub multiple_outputs: bool,
    pub multi_window: bool,
    pub midi: bool,
    pub mic: bool,
    pub background_audio: bool,
    pub prelisten: bool,
}

#[tauri::command]
pub fn get_platform_capabilities() -> Capabilities {
    let android = cfg!(target_os = "android");
    Capabilities {
        platform: if android { "android" } else { "desktop" },
        multiple_outputs: !android,
        multi_window: !android,
        midi: cfg!(any(target_os = "windows", target_os = "linux")),
        mic: !android,
        background_audio: !android,
        prelisten: !android,
    }
}

pub fn initialize(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;
        android_audio::initialize()?;
        let path = app.path().app_data_dir()?;
        std::fs::create_dir_all(&path)?;
        DATA_DIR
            .set(path)
            .map_err(|_| "platform already initialized")?;
    }
    crate::engine::files::initialize(app.clone());
    crate::engine::backup::recover_pending_restore()?;
    crate::engine::library::file_rename::recover_pending()?;
    crate::engine::library::tag_write_journal::recover()?;
    Ok(())
}

#[cfg(target_os = "android")]
pub fn data_dir() -> PathBuf {
    DATA_DIR
        .get()
        .expect("private directory initialized before AppState")
        .clone()
}
