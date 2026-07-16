//! Modulo: cmd_button_color.rs
//! Proposito: pintar de un color VARIOS botones a la vez (seleccion multiple con
//! Ctrl+clic). Sirve a la rejilla y al panel fijo, que es donde el usuario puede
//! seleccionar.
//!
//! Existe en vez de una "politica de colores" configurable: se descarto por
//! complicada de explicar y de usar. Los botones nuevos siguen saliendo en
//! aleatorio; si el operador quiere ordenar la botonera, selecciona y pinta. El
//! resultado se ve al momento, sin configurar nada por adelantado.
use super::cmd_fixed_panel::buttons_mut;
use super::AppState;
use crate::domain::colors::text_for_theme;
use crate::engine::persist::config_io;
use crate::model::{AppConfig, ButtonData};

/// Pinta los botones indicados. `group` distingue de donde salen: "grid" (la
/// paleta activa) o "fixed" (el panel, en el alcance vigente).
///
/// El color del TEXTO no se pide: lo calcula Rust para que se lea sobre el fondo
/// en el tema actual (regla 8, misma via que al crear un boton).
#[tauri::command]
pub fn set_buttons_color(
    indexes: Vec<u32>,
    color_bg: String,
    group: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if indexes.is_empty() {
        return Err("no_buttons_selected".into());
    }
    let mut cfg = state.config.lock().unwrap();
    let color_text = text_for_theme(&color_bg, &cfg.theme.clone(), "button");
    let painted = paint(&mut cfg, &indexes, &color_bg, &color_text, &group)?;
    if painted == 0 {
        return Err("button_not_found".into());
    }
    config_io::save_config(&cfg)
}

/// Devuelve cuantos se pintaron. Un indice que ya no existe se ignora en vez de
/// abortar: entre la seleccion y el clic derecho la rejilla pudo cambiar, y
/// perder el trabajo por una celda vieja seria peor que pintar el resto.
fn paint(
    cfg: &mut AppConfig,
    indexes: &[u32],
    color_bg: &str,
    color_text: &str,
    group: &str,
) -> Result<usize, String> {
    let buttons: &mut Vec<ButtonData> = match group {
        "fixed" => buttons_mut(cfg)?,
        "grid" => {
            let paleta_id = cfg.active_profile().map(|p| p.active_paleta_id.clone());
            let id = paleta_id.ok_or("active_profile_not_found")?;
            &mut cfg
                .active_profile_mut()
                .ok_or("active_profile_not_found")?
                .paletas
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or("paleta_not_found")?
                .botones
        }
        _ => return Err("invalid_button_group".into()),
    };
    let mut painted = 0;
    for btn in buttons.iter_mut().filter(|b| indexes.contains(&b.index)) {
        btn.color_bg = color_bg.to_string();
        btn.color_text = color_text.to_string();
        painted += 1;
    }
    Ok(painted)
}

#[cfg(test)]
#[path = "cmd_button_color_tests.rs"]
mod cmd_button_color_tests;
