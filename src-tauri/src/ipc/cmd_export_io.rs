use super::AppState;
use crate::domain::export::{lfa_format::LfaProfile, tracks as export_tracks};
use serde_json::Value;

/// Inyecta `bdelf_tracks`, serializa y abre el diálogo de guardado.
pub(super) fn finish_export(
    state: &AppState,
    label: &str,
    ext: &str,
    name: String,
    mut value: Value,
    paths: Vec<String>,
) -> Result<(), String> {
    {
        let store = state.tracks.lock().unwrap();
        export_tracks::inject(&mut value, &store, &paths);
    }
    let json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    write_export(label, ext, &name, &json)
}

fn write_export(label: &str, extension: &str, name: &str, json: &str) -> Result<(), String> {
    let path = crate::engine::files::FileDialog::new()
        .add_filter(label, &[extension])
        .set_file_name(&format!("{}.{}", name, extension))
        .save_file()?
        .ok_or("Operación cancelada.")?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    crate::engine::files::commit_export(&path)
}

/// Acepta un perfil suelto o un estado completo { activeProfileId, profiles }.
pub(super) fn parse_profile(value: Value) -> Result<LfaProfile, String> {
    if let Some(profiles) = value.get("profiles").and_then(|p| p.as_array()) {
        let active_id = value
            .get("activeProfileId")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let chosen = profiles
            .iter()
            .find(|p| p.get("id").and_then(|v| v.as_str()) == Some(active_id))
            .or_else(|| profiles.first())
            .ok_or("El archivo no contiene perfiles")?;
        return serde_json::from_value(chosen.clone()).map_err(|e| e.to_string());
    }
    serde_json::from_value(value).map_err(|e| e.to_string())
}
