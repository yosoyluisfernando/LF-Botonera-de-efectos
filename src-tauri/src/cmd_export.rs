/// Módulo: cmd_export.rs
/// Propósito: Comandos IPC de exportación/importación .bdelf (pestaña) y
/// .bdeplf (perfil). El formato y las conversiones viven en lfa_format.rs.
/// Igual que el LFA, el import de perfil acepta tanto un perfil suelto como
/// un estado completo { activeProfileId, profiles: [...] }.

use super::AppState;
use crate::config;
use crate::lfa_format::{self, LfaPaleta, LfaProfile};
use crate::types::GridState;

/// Exporta la pestaña activa como .bdelf (formato maqueta).
#[tauri::command]
pub fn export_tab(state: tauri::State<AppState>) -> Result<(), String> {
    let cfg     = state.config.lock().unwrap();
    let pid     = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter().find(|p| p.id == pid).ok_or("Perfil no encontrado")?;
    let aid     = profile.active_paleta_id.clone();
    let paleta  = profile.paletas.iter().find(|p| p.id == aid).ok_or("Pestaña no encontrada")?;
    let lfa     = lfa_format::to_lfa_paleta(paleta);
    let nombre  = paleta.nombre.clone();
    drop(cfg);
    let json = serde_json::to_string_pretty(&lfa).map_err(|e| e.to_string())?;
    let path = rfd::FileDialog::new()
        .add_filter("LF Botonera Tab", &["bdelf"])
        .set_file_name(&format!("{}.bdelf", nombre))
        .save_file().ok_or("Operación cancelada.")?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

/// Importa un .bdelf como nueva pestaña del perfil activo.
#[tauri::command]
pub fn import_tab(state: tauri::State<AppState>) -> Result<GridState, String> {
    let pick = rfd::FileDialog::new()
        .add_filter("LF Botonera Tab", &["bdelf"])
        .pick_file().ok_or("Operación cancelada.")?;
    let json    = std::fs::read_to_string(pick).map_err(|e| e.to_string())?;
    let lfa: LfaPaleta = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let mut cfg = state.config.lock().unwrap();
    let pid     = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter_mut().find(|p| p.id == pid).ok_or("Perfil no encontrado")?;
    let new_id  = format!("paleta_imp_{}", profile.paletas.len() + 1);
    let paleta  = lfa_format::from_lfa_paleta(lfa, new_id.clone());
    let grid    = GridState { columns: paleta.cols, rows: paleta.rows, buttons: paleta.botones.clone() };
    profile.paletas.push(paleta);
    profile.active_paleta_id = new_id;
    config::save_config(&cfg)?;
    Ok(grid)
}

/// Exporta el perfil activo como .bdeplf (incluye config: salidas y atajos).
#[tauri::command]
pub fn export_profile(state: tauri::State<AppState>) -> Result<(), String> {
    let cfg     = state.config.lock().unwrap();
    let pid     = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter().find(|p| p.id == pid).ok_or("Perfil no encontrado")?;
    let lfa     = lfa_format::to_lfa_profile(profile);
    let nombre  = profile.name.clone();
    drop(cfg);
    let json = serde_json::to_string_pretty(&lfa).map_err(|e| e.to_string())?;
    let path = rfd::FileDialog::new()
        .add_filter("LF Botonera Profile", &["bdeplf"])
        .set_file_name(&format!("{}.bdeplf", nombre))
        .save_file().ok_or("Operación cancelada.")?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

/// Importa un .bdeplf como nuevo perfil y lo activa.
#[tauri::command]
pub fn import_profile(state: tauri::State<AppState>) -> Result<(), String> {
    let pick = rfd::FileDialog::new()
        .add_filter("LF Botonera Profile", &["bdeplf"])
        .pick_file().ok_or("Operación cancelada.")?;
    let json = std::fs::read_to_string(pick).map_err(|e| e.to_string())?;
    let lfa  = _parse_profile(&json)?;
    if lfa.paletas.is_empty() {
        return Err("El perfil importado no contiene pestañas".to_string());
    }
    let mut cfg = state.config.lock().unwrap();
    let new_id  = format!("perfil_imp_{}", cfg.profiles.len() + 1);
    let profile = lfa_format::from_lfa_profile(lfa, new_id.clone());
    cfg.profiles.push(profile);
    cfg.active_profile_id = new_id;
    config::save_config(&cfg)
}

/// Acepta un perfil suelto o un estado completo { activeProfileId, profiles }.
/// Con estado completo se toma el perfil activo (o el primero), como el LFA.
fn _parse_profile(json: &str) -> Result<LfaProfile, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if let Some(profiles) = value.get("profiles").and_then(|p| p.as_array()) {
        let active_id = value.get("activeProfileId").and_then(|v| v.as_str()).unwrap_or("");
        let chosen = profiles.iter()
            .find(|p| p.get("id").and_then(|v| v.as_str()) == Some(active_id))
            .or_else(|| profiles.first())
            .ok_or("El archivo no contiene perfiles")?;
        return serde_json::from_value(chosen.clone()).map_err(|e| e.to_string());
    }
    serde_json::from_value(value).map_err(|e| e.to_string())
}
