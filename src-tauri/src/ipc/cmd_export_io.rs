/// Preparación de payloads y archivos .bdelf/.bdeplf para los comandos de exportación.
use crate::core::AppState;
use crate::domain::export::lfa_format::{self, LfaProfile};
use crate::domain::export::tracks as export_tracks;
use crate::model::{AppConfig, ProfileData};
use serde_json::Value;

pub(super) fn active_profile(cfg: &AppConfig) -> Result<&ProfileData, String> {
    profile_by_id(cfg, &cfg.active_profile_id)
}

pub(super) fn profile_by_id<'a>(cfg: &'a AppConfig, id: &str) -> Result<&'a ProfileData, String> {
    cfg.profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or("Perfil no encontrado".to_string())
}

pub(super) fn paleta_export_payload(
    paleta_id: &str,
    profile: &ProfileData,
) -> Result<(String, Value, Vec<String>), String> {
    let paleta = profile
        .paletas
        .iter()
        .find(|p| p.id == paleta_id)
        .ok_or("Pestaña no encontrada")?;
    let value =
        serde_json::to_value(lfa_format::to_lfa_paleta(paleta)).map_err(|e| e.to_string())?;
    Ok((
        paleta.nombre.clone(),
        value,
        export_tracks::paleta_paths(paleta),
    ))
}

pub(super) fn profile_export_payload(
    profile: &ProfileData,
) -> Result<(String, Value, Vec<String>), String> {
    let value =
        serde_json::to_value(lfa_format::to_lfa_profile(profile)).map_err(|e| e.to_string())?;
    let paths = profile
        .paletas
        .iter()
        .flat_map(export_tracks::paleta_paths)
        .collect();
    Ok((profile.name.clone(), value, paths))
}

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
    let path = rfd::FileDialog::new()
        .add_filter(label, &[extension])
        .set_file_name(&format!("{}.{}", name, extension))
        .save_file()
        .ok_or("Operación cancelada.")?;
    std::fs::write(path, json).map_err(|e| e.to_string())
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
