//! Conversión auxiliar basada en la aportación de Danilo Wenner.
//! Una cola Rust prepara copias FLAC; el original conserva cue, ganancia e identidad.
//! Android mantiene su decodificación nativa, sin procesos externos.
#[cfg(not(target_os = "android"))]
mod cache;
#[cfg(not(target_os = "android"))]
mod ffmpeg;
#[cfg(not(target_os = "android"))]
mod probe;
pub mod progress;
#[cfg(not(target_os = "android"))]
mod state;
#[cfg(not(target_os = "android"))]
mod worker;

pub fn set_app(app: tauri::AppHandle) {
    progress::set_app(app);
}
pub fn prepare(path: &str) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    if probe::is_candidate(path) {
        return worker::prepare(path);
    }
    let _ = path;
    Ok(())
}
pub fn resolve(path: &str) -> Result<String, String> {
    #[cfg(not(target_os = "android"))]
    if probe::is_candidate(path) {
        return worker::resolve(path, false);
    }
    Ok(path.to_string())
}
/// Exclusivo de los workers de análisis: puede esperar la conversión en cola.
pub fn for_analysis(path: &str) -> Result<String, String> {
    #[cfg(not(target_os = "android"))]
    if probe::is_candidate(path) {
        return worker::resolve(path, true);
    }
    Ok(path.to_string())
}
pub fn is_candidate(path: &str) -> bool {
    #[cfg(not(target_os = "android"))]
    return probe::is_candidate(path);
    #[cfg(target_os = "android")]
    {
        let _ = path;
        false
    }
}

#[cfg(all(test, not(target_os = "android")))]
mod integration_tests;

#[cfg(all(test, not(target_os = "android")))]
mod real_file_tests;

#[cfg(all(test, not(target_os = "android")))]
mod native_file_tests;
