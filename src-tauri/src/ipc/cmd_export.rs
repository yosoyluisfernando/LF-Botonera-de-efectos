use super::cmd_export_io::{finish_export, paleta_export_payload, parse_profile};
use super::cmd_profiles::next_id;
/// Modulo: cmd_export.rs
/// Proposito: comandos IPC de exportacion/importacion .bdelf y .bdeplf. Ademas
/// de la pestaña/perfil, empaqueta los metadatos del editor (cue, dB) en un
/// campo opcional `bdelf_tracks` para que no se pierdan (ver export_tracks.rs).
use super::AppState;
use crate::domain::export::lfa_format::{self, LfaPaleta};
use crate::domain::export::tracks as export_tracks;
use crate::domain::grid::view::paleta_to_grid;
use crate::engine::persist::config_io as config;
use crate::model::grid::GridState;
use crate::model::{AppConfig, ProfileData};

use serde_json::Value;

/// Exporta la pestaña activa como .bdelf.
#[tauri::command(async)]
pub fn export_tab(state: tauri::State<AppState>) -> Result<(), String> {
    let (name, value, paths) = {
        let cfg = state.config.lock().unwrap();
        let profile = active_profile(&cfg)?;
        paleta_export_payload(&profile.active_paleta_id, profile)?
    };
    finish_export(
        &state,
        "LF Botonera de Efectos Tab",
        "bdelf",
        name,
        value,
        paths,
    )
}

/// Exporta una pestaña concreta sin cambiar el estado activo de la aplicacion.
#[tauri::command(async)]
pub fn export_tab_by_id(
    profile_id: String,
    paleta_id: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let (name, value, paths) = {
        let cfg = state.config.lock().unwrap();
        let profile = profile_by_id(&cfg, &profile_id)?;
        paleta_export_payload(&paleta_id, profile)?
    };
    finish_export(
        &state,
        "LF Botonera de Efectos Tab",
        "bdelf",
        name,
        value,
        paths,
    )
}

/// Importa un .bdelf como nueva pestaña del perfil activo.
#[tauri::command(async)]
pub fn import_tab(state: tauri::State<AppState>) -> Result<GridState, String> {
    let pick = crate::engine::files::FileDialog::new()
        .add_filter("LF Botonera de Efectos Tab", &["bdelf"])
        .pick_file()?
        .ok_or("Operación cancelada.")?;
    let json = std::fs::read_to_string(pick).map_err(|e| e.to_string())?;
    let mut value: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let tracks = value.as_object_mut().and_then(|o| o.remove("bdelf_tracks"));
    let lfa: LfaPaleta = serde_json::from_value(value).map_err(|e| e.to_string())?;
    let grid = {
        let mut cfg = state.config.lock().unwrap();
        let pid = cfg.active_profile_id.clone();
        let ids: Vec<_> = cfg.profiles.iter().flat_map(|p| &p.paletas)
            .map(|p| p.id.clone()).collect();
        let new_id = next_id("paleta_imp", &ids);
        let profile = cfg
            .profiles
            .iter_mut()
            .find(|p| p.id == pid)
            .ok_or("Perfil no encontrado")?;
        let paleta = lfa_format::from_lfa_paleta(lfa, new_id.clone());
        if cfg!(target_os = "android") && paleta.obs {
            return Err("platform_unavailable".into());
        }
        crate::domain::obs::validate_paleta(&paleta)?;
        let grid = paleta_to_grid(&paleta);
        profile.paletas.push(paleta);
        profile.active_paleta_id = new_id;
        config::save_config(&cfg)?;
        grid
    };
    if let Some(t) = tracks {
        export_tracks::restore(&state.tracks.lock().unwrap(), &t);
    }
    Ok(grid)
}

/// Exporta el perfil activo como .bdeplf.
#[tauri::command(async)]
pub fn export_profile(state: tauri::State<AppState>) -> Result<(), String> {
    let (name, value, paths) = {
        let cfg = state.config.lock().unwrap();
        profile_export_payload(active_profile(&cfg)?)?
    };
    finish_export(
        &state,
        "LF Botonera de Efectos Profile",
        "bdeplf",
        name,
        value,
        paths,
    )
}

/// Exporta un perfil concreto sin cambiar el perfil activo.
#[tauri::command(async)]
pub fn export_profile_by_id(
    profile_id: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let (name, value, paths) = {
        let cfg = state.config.lock().unwrap();
        profile_export_payload(profile_by_id(&cfg, &profile_id)?)?
    };
    finish_export(
        &state,
        "LF Botonera de Efectos Profile",
        "bdeplf",
        name,
        value,
        paths,
    )
}

/// Importa un .bdeplf como nuevo perfil y lo activa.
#[tauri::command(async)]
pub fn import_profile(state: tauri::State<AppState>) -> Result<(), String> {
    let pick = crate::engine::files::FileDialog::new()
        .add_filter("LF Botonera de Efectos Profile", &["bdeplf"])
        .pick_file()?
        .ok_or("Operación cancelada.")?;
    let json = std::fs::read_to_string(pick).map_err(|e| e.to_string())?;
    let mut value: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let tracks = value.as_object_mut().and_then(|o| o.remove("bdelf_tracks"));
    let lfa = parse_profile(value)?;
    if lfa.paletas.is_empty() {
        return Err("El perfil importado no contiene pestañas".to_string());
    }
    {
        let mut cfg = state.config.lock().unwrap();
        let ids: Vec<_> = cfg.profiles.iter().map(|p| p.id.clone()).collect();
        let new_id = next_id("perfil_imp", &ids);
        let profile = lfa_format::from_lfa_profile(lfa, new_id.clone());
        if cfg!(target_os = "android") && profile.paletas.iter().any(|paleta| paleta.obs) {
            return Err("platform_unavailable".into());
        }
        crate::domain::obs::validate_profile(&profile)?;
        cfg.profiles.push(profile);
        cfg.active_profile_id = new_id;
        config::save_config(&cfg)?;
    }
    if let Some(t) = tracks {
        export_tracks::restore(&state.tracks.lock().unwrap(), &t);
    }
    let id = state.config.lock().unwrap().active_profile_id.clone();
    super::cmd_profiles::set_active_profile(id, state)?;
    Ok(())
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

fn profile_export_payload(profile: &ProfileData) -> Result<(String, Value, Vec<String>), String> {
    let value =
        serde_json::to_value(lfa_format::to_lfa_profile(profile)).map_err(|e| e.to_string())?;
    let paths = profile
        .paletas
        .iter()
        .flat_map(export_tracks::paleta_paths)
        .collect();
    Ok((profile.name.clone(), value, paths))
}
