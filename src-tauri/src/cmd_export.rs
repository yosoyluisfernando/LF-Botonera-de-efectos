/// Modulo: cmd_export.rs
/// Proposito: comandos IPC de exportacion/importacion .bdelf y .bdeplf.
use super::AppState;
use crate::config;
use crate::grid_view::paleta_to_grid;
use crate::lfa_format::{self, LfaPaleta, LfaProfile};
use crate::types::{AppConfig, ProfileData};
use crate::types_grid::GridState;

/// Exporta la pestaña activa como .bdelf.
#[tauri::command]
pub fn export_tab(state: tauri::State<AppState>) -> Result<(), String> {
    let (name, json) = {
        let cfg = state.config.lock().unwrap();
        let profile = active_profile(&cfg)?;
        paleta_export_payload(&profile.active_paleta_id, profile)?
    };
    write_export("LF Botonera de Efectos Tab", "bdelf", &name, &json)
}

/// Exporta una pestaña concreta sin cambiar el estado activo de la aplicacion.
#[tauri::command]
pub fn export_tab_by_id(
    profile_id: String,
    paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let (name, json) = {
        let cfg = state.config.lock().unwrap();
        let profile = profile_by_id(&cfg, &profile_id)?;
        paleta_export_payload(&paleta_id, profile)?
    };
    write_export("LF Botonera de Efectos Tab", "bdelf", &name, &json)
}

/// Importa un .bdelf como nueva pestaña del perfil activo.
#[tauri::command]
pub fn import_tab(state: tauri::State<AppState>) -> Result<GridState, String> {
    let pick = rfd::FileDialog::new()
        .add_filter("LF Botonera de Efectos Tab", &["bdelf"])
        .pick_file()
        .ok_or("Operación cancelada.")?;
    let json = std::fs::read_to_string(pick).map_err(|e| e.to_string())?;
    let lfa: LfaPaleta = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let mut cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil no encontrado")?;
    let new_id = format!("paleta_imp_{}", profile.paletas.len() + 1);
    let paleta = lfa_format::from_lfa_paleta(lfa, new_id.clone());
    let grid = paleta_to_grid(&paleta);
    profile.paletas.push(paleta);
    profile.active_paleta_id = new_id;
    config::save_config(&cfg)?;
    Ok(grid)
}

/// Exporta el perfil activo como .bdeplf.
#[tauri::command]
pub fn export_profile(state: tauri::State<AppState>) -> Result<(), String> {
    let (name, json) = {
        let cfg = state.config.lock().unwrap();
        profile_export_payload(active_profile(&cfg)?)?
    };
    write_export("LF Botonera de Efectos Profile", "bdeplf", &name, &json)
}

/// Exporta un perfil concreto sin cambiar el perfil activo.
#[tauri::command]
pub fn export_profile_by_id(
    profile_id: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let (name, json) = {
        let cfg = state.config.lock().unwrap();
        profile_export_payload(profile_by_id(&cfg, &profile_id)?)?
    };
    write_export("LF Botonera de Efectos Profile", "bdeplf", &name, &json)
}

/// Importa un .bdeplf como nuevo perfil y lo activa.
#[tauri::command]
pub fn import_profile(state: tauri::State<AppState>) -> Result<(), String> {
    let pick = rfd::FileDialog::new()
        .add_filter("LF Botonera de Efectos Profile", &["bdeplf"])
        .pick_file()
        .ok_or("Operación cancelada.")?;
    let json = std::fs::read_to_string(pick).map_err(|e| e.to_string())?;
    let lfa = parse_profile(&json)?;
    if lfa.paletas.is_empty() {
        return Err("El perfil importado no contiene pestañas".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    let new_id = format!("perfil_imp_{}", cfg.profiles.len() + 1);
    let profile = lfa_format::from_lfa_profile(lfa, new_id.clone());
    cfg.profiles.push(profile);
    cfg.active_profile_id = new_id;
    config::save_config(&cfg)
}

fn active_profile(cfg: &AppConfig) -> Result<&ProfileData, String> {
    profile_by_id(cfg, &cfg.active_profile_id)
}

fn profile_by_id<'a>(cfg: &'a AppConfig, id: &str) -> Result<&'a ProfileData, String> {
    cfg.profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or("Perfil no encontrado".to_string())
}

fn paleta_export_payload(
    paleta_id: &str,
    profile: &ProfileData,
) -> Result<(String, String), String> {
    let paleta = profile
        .paletas
        .iter()
        .find(|p| p.id == paleta_id)
        .ok_or("Pestaña no encontrada")?;
    let lfa = lfa_format::to_lfa_paleta(paleta);
    let json = serde_json::to_string_pretty(&lfa).map_err(|e| e.to_string())?;
    Ok((paleta.nombre.clone(), json))
}

fn profile_export_payload(profile: &ProfileData) -> Result<(String, String), String> {
    let lfa = lfa_format::to_lfa_profile(profile);
    let json = serde_json::to_string_pretty(&lfa).map_err(|e| e.to_string())?;
    Ok((profile.name.clone(), json))
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
fn parse_profile(json: &str) -> Result<LfaProfile, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
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
