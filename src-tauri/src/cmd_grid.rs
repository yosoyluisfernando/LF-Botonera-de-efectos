/// Módulo: cmd_grid.rs
/// Propósito: Comandos IPC para gestionar botones dentro de la pestaña activa.

use super::AppState;
use crate::cmd_audio::probe_duration_secs;
use crate::colors::random_color;
use crate::config;
use crate::types::{AppConfig, ButtonData, GridState, PaletaData};

// ─── Helper ───────────────────────────────────────────────────────────────────

/// Devuelve una referencia mutable a la paleta activa del perfil activo.
pub(crate) fn active_paleta(cfg: &mut AppConfig) -> Result<&mut PaletaData, String> {
    let pid = cfg.active_profile_id.clone();
    let profile = cfg.profiles.iter_mut()
        .find(|p| p.id == pid)
        .ok_or("Perfil activo no encontrado")?;
    let aid = profile.active_paleta_id.clone();
    profile.paletas.iter_mut()
        .find(|p| p.id == aid)
        .ok_or("Pestaña activa no encontrada".to_string())
}

pub(crate) fn paleta_to_grid(p: &PaletaData) -> GridState {
    GridState { columns: p.cols, rows: p.rows, buttons: p.botones.clone() }
}

// ─── Comandos ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_grid_state(state: tauri::State<AppState>) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta  = active_paleta(&mut cfg)?;
    // Re-sondear duraciones faltantes (botones guardados antes de esta versión).
    // duration = -1 marca "sondeo fallido" para no reintentar en cada refresco.
    let mut changed = false;
    for b in paleta.botones.iter_mut() {
        if b.duration == 0.0 && !b.path.is_empty() {
            let secs = probe_duration_secs(&b.path);
            if secs > 0.0 {
                b.duration     = secs;
                b.duration_str = format!("{:.1}s", secs);
                changed = true;
            } else {
                b.duration = -1.0;
            }
        }
    }
    let grid = paleta_to_grid(paleta);
    if changed { config::save_config(&cfg)?; }
    Ok(grid)
}

/// Asigna un archivo de audio a un botón. Si `path` es None abre el explorador.
#[tauri::command]
pub fn assign_file_to_button(
    index: u32,
    path: Option<String>,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let file_path = match path {
        Some(p) => p,
        None => rfd::FileDialog::new()
            .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a", "wma"])
            .pick_file()
            .map(|p| p.to_string_lossy().to_string())
            .ok_or("Operación cancelada.")?,
    };
    let stem = std::path::Path::new(&file_path)
        .file_stem().unwrap_or_default()
        .to_string_lossy().to_uppercase();

    let secs = probe_duration_secs(&file_path);
    let (duration, duration_str) = if secs > 0.0 {
        (secs, format!("{:.1}s", secs))
    } else {
        (-1.0, String::new())
    };

    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    // Id único por pestaña: evita colisiones de audio entre pestañas
    let btn_id = format!("{}_btn_{}", paleta.id, index);
    paleta.botones.push(ButtonData {
        id: btn_id, index,
        label: stem.clone(), type_field: "audio".to_string(),
        path: file_path, folder: String::new(), name: stem.to_string(),
        color_bg: random_color(), color_text: "#FFFFFF".to_string(),
        vol: 1.0, duration, duration_str,
        loop_mode: false, stop_other: false, overlap: false,
        restart: false, shortcut: String::new(),
    });
    let grid = paleta_to_grid(paleta);
    config::save_config(&cfg)?;
    Ok(grid)
}

/// Elimina el botón del índice indicado y persiste el estado.
#[tauri::command]
pub fn clear_button(index: u32, state: tauri::State<AppState>) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta = active_paleta(&mut cfg)?;
    paleta.botones.retain(|b| b.index != index);
    let grid = paleta_to_grid(paleta);
    config::save_config(&cfg)?;
    Ok(grid)
}

/// Intercambia dos botones por índice (Alt+arrastrar). Si el destino está vacío, mueve.
#[tauri::command]
pub fn reorder_buttons(
    from_index: u32,
    to_index:   u32,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta  = active_paleta(&mut cfg)?;
    let pid     = paleta.id.clone();
    let pos_src = paleta.botones.iter().position(|b| b.index == from_index);
    let pos_dst = paleta.botones.iter().position(|b| b.index == to_index);
    match (pos_src, pos_dst) {
        (Some(a), Some(b)) => {
            paleta.botones[a].index = to_index;
            paleta.botones[a].id    = format!("{}_btn_{}", pid, to_index);
            paleta.botones[b].index = from_index;
            paleta.botones[b].id    = format!("{}_btn_{}", pid, from_index);
        }
        (Some(a), None) => {
            paleta.botones[a].index = to_index;
            paleta.botones[a].id    = format!("{}_btn_{}", pid, to_index);
        }
        _ => {}
    }
    let grid = paleta_to_grid(paleta);
    config::save_config(&cfg)?;
    Ok(grid)
}

/// Actualiza (o CREA, si la celda estaba vacía) un botón con todos sus campos.
/// `btn_type`: "audio" | "time" | "temperature" | "humidity" (estilo LFA).
/// Para locuciones, `folder` lleva la carpeta propia del botón y `path` queda vacío.
#[tauri::command]
pub fn update_button_data(
    index:      u32,
    label:      String,
    color_bg:   String,
    color_text: String,
    btn_type:   Option<String>,
    path:       Option<String>,
    folder:     Option<String>,
    vol:        Option<f32>,
    loop_mode:  Option<bool>,
    stop_other: Option<bool>,
    overlap:    Option<bool>,
    restart:    Option<bool>,
    shortcut:   Option<String>,
    state: tauri::State<AppState>,
) -> Result<GridState, String> {
    let mut cfg = state.config.lock().unwrap();
    let paleta  = active_paleta(&mut cfg)?;
    let pid     = paleta.id.clone();

    // Upsert: si la celda estaba vacía se crea el botón (ej. locución nueva)
    if !paleta.botones.iter().any(|b| b.index == index) {
        paleta.botones.push(ButtonData {
            id: format!("{}_btn_{}", pid, index), index,
            label: label.clone(), type_field: "audio".to_string(),
            path: String::new(), folder: String::new(), name: label.clone(),
            color_bg: color_bg.clone(), color_text: color_text.clone(),
            vol: 1.0, duration: -1.0, duration_str: String::new(),
            loop_mode: false, stop_other: false, overlap: false,
            restart: false, shortcut: String::new(),
        });
    }
    let btn = paleta.botones.iter_mut().find(|b| b.index == index).unwrap();

    // name y label se actualizan juntos: la celda muestra name primero
    btn.label      = label.clone();
    btn.name       = label;
    btn.color_bg   = color_bg;
    btn.color_text = color_text;
    if let Some(v) = btn_type { btn.type_field = v; }
    if let Some(v) = path {
        if v != btn.path {
            // Ruta nueva: duración a 0 para que get_grid_state la re-sondee
            btn.duration     = 0.0;
            btn.duration_str = String::new();
        }
        btn.path = v;
    }
    if let Some(v) = folder     { btn.folder     = v; }
    if let Some(v) = vol        { btn.vol        = v; }
    if let Some(v) = loop_mode  { btn.loop_mode  = v; }
    if let Some(v) = stop_other { btn.stop_other = v; }
    if let Some(v) = overlap    { btn.overlap    = v; }
    if let Some(v) = restart    { btn.restart    = v; }
    if let Some(v) = shortcut   { btn.shortcut   = v; }

    let grid = paleta_to_grid(paleta);
    config::save_config(&cfg)?;
    Ok(grid)
}
