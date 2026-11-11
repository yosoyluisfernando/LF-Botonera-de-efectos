//! Diseño visual opcional para Windows.
use super::AppState;
use crate::engine::persist::config_io;
use crate::model::appearance::DesktopStyle;
use serde::Serialize;
use tauri::Emitter;

const AVAILABLE: bool = cfg!(target_os = "windows");

#[derive(Clone, Serialize)]
pub struct AppearanceView {
    available: bool,
    style: DesktopStyle,
}

#[tauri::command]
pub fn get_desktop_appearance(state: tauri::State<AppState>) -> AppearanceView {
    AppearanceView {
        available: AVAILABLE,
        style: if AVAILABLE {
            state.config.lock().unwrap().desktop_style
        } else {
            DesktopStyle::Default
        },
    }
}

#[tauri::command]
pub fn set_desktop_appearance(
    style: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<AppearanceView, String> {
    if !AVAILABLE {
        return Err("desktop_style_unavailable".into());
    }
    let style = DesktopStyle::parse(&style)?;
    {
        let mut cfg = state.config.lock().unwrap();
        let previous = cfg.desktop_style;
        cfg.desktop_style = style;
        if let Err(error) = config_io::save_config(&cfg) {
            cfg.desktop_style = previous;
            return Err(error);
        }
    }
    let view = AppearanceView {
        available: true,
        style,
    };
    // El comando devuelve la verdad persistida incluso si una ventana ya cerró.
    if let Err(error) = app.emit("desktop-appearance-changed", &view) {
        eprintln!("desktop-appearance-changed: {error}");
    }
    Ok(view)
}
