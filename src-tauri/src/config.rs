use crate::types::{AppConfig, ButtonData};
/// Módulo: config.rs
/// Propósito: Persistencia de AppConfig en disco con migración automática
/// desde el formato antiguo (audio_device plano + grid_state.json separado).
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

// ─── Rutas ────────────────────────────────────────────────────────────────────

/// Windows: %APPDATA%\LF Botonera\   Linux: ~/.config/LF Botonera/
pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    base.join("LF Botonera")
}

// ─── Formatos legados (sólo para migración) ───────────────────────────────────

#[derive(Deserialize)]
struct LegacyConfig {
    is_first_boot: Option<bool>,
    weather_module_enabled: Option<bool>,
    lf_automatizador_link: Option<bool>,
    theme: Option<String>,
    audio_device: Option<String>,
}

#[derive(Deserialize)]
struct LegacyGrid {
    columns: Option<u32>,
    rows: Option<u32>,
    buttons: Option<Vec<LegacyButton>>,
}

#[derive(Deserialize)]
struct LegacyButton {
    id: String,
    index: u32,
    label: String,
    path: String,
    color_bg: String,
    color_text: String,
    duration_str: Option<String>,
}

// ─── Carga ────────────────────────────────────────────────────────────────────

pub fn load_config() -> AppConfig {
    let path = get_data_dir().join("botonera_config.json");
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return AppConfig::default(),
    };

    // Intentar formato nuevo primero
    if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&raw) {
        if !cfg.profiles.is_empty() {
            normalize_button_ids(&mut cfg);
            normalize_playback_modes(&mut cfg);
            return cfg;
        }
    }

    // Migrar desde formato antiguo
    if let Ok(old) = serde_json::from_str::<LegacyConfig>(&raw) {
        return migrate(old);
    }

    AppConfig::default()
}

fn migrate(old: LegacyConfig) -> AppConfig {
    let mut cfg = AppConfig::default();
    if let Some(v) = old.is_first_boot {
        cfg.is_first_boot = v;
    }
    if let Some(v) = old.weather_module_enabled {
        cfg.weather_module_enabled = v;
    }
    if let Some(v) = old.lf_automatizador_link {
        cfg.lf_automatizador_link = v;
    }
    if let Some(v) = old.theme {
        cfg.theme = v;
    }

    if let Some(profile) = cfg.profiles.first_mut() {
        if let Some(dev) = old.audio_device {
            profile.audio.out_main = dev;
        }
        if let Some(paleta) = profile.paletas.first_mut() {
            if let Some(grid) = load_legacy_grid() {
                paleta.rows = grid.rows.unwrap_or(5);
                paleta.cols = grid.columns.unwrap_or(5);
                paleta.botones = grid
                    .buttons
                    .unwrap_or_default()
                    .into_iter()
                    .map(|b| ButtonData {
                        id: b.id,
                        index: b.index,
                        label: b.label.clone(),
                        type_field: "audio".to_string(),
                        path: b.path,
                        folder: String::new(),
                        name: b.label,
                        color_bg: b.color_bg,
                        color_text: b.color_text,
                        vol: 1.0,
                        duration: 0.0,
                        duration_str: b.duration_str.unwrap_or_default(),
                        loop_mode: false,
                        stop_other: false,
                        overlap: false,
                        restart: false,
                        shortcut: String::new(),
                    })
                    .collect();
            }
        }
    }
    cfg
}

/// Garantiza que los ids de botón sean únicos entre pestañas
/// (formato "{paleta_id}_btn_{index}"). Migra configs con el formato
/// antiguo "btn_{index}", que colisionaba entre pestañas.
fn normalize_button_ids(cfg: &mut AppConfig) {
    for profile in cfg.profiles.iter_mut() {
        for paleta in profile.paletas.iter_mut() {
            let pid = paleta.id.clone();
            for b in paleta.botones.iter_mut() {
                let expected = format!("{}_btn_{}", pid, b.index);
                if b.id != expected {
                    b.id = expected;
                }
            }
        }
    }
}

fn normalize_playback_modes(cfg: &mut AppConfig) {
    for profile in cfg.profiles.iter_mut() {
        if profile.audio.playback_mode == "stop_others" {
            profile.audio.playback_mode = "normal".to_string();
            profile.audio.solo_mode = true;
        }
    }
}

fn load_legacy_grid() -> Option<LegacyGrid> {
    let path = get_data_dir().join("grid_state.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
}

// ─── Guardado ─────────────────────────────────────────────────────────────────

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = get_data_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(dir.join("botonera_config.json"), data).map_err(|e| e.to_string())
}
