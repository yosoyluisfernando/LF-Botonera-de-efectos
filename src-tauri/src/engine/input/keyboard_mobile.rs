//! Android no registra atajos globales; conserva las asignaciones importadas.
pub fn sync(_: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
}
