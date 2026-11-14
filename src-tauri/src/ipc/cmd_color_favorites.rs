//! Favoritos globales. Se guardan con la configuración normal y sus respaldos.
use super::AppState;
use crate::engine::persist::config_io;
use crate::model::color_favorite::{self, ColorFavorite};
#[tauri::command]
pub fn get_color_favorites(state: tauri::State<AppState>) -> Vec<ColorFavorite> {
    state.config.lock().unwrap().color_favorites.clone()
}
#[tauri::command]
pub fn save_color_favorite(
    favorite: ColorFavorite,
    state: tauri::State<AppState>,
) -> Result<Vec<ColorFavorite>, String> {
    let mut cfg = state.config.lock().unwrap();
    let mut next = cfg.clone();
    color_favorite::add(&mut next.color_favorites, favorite)?;
    config_io::save_config(&next)?;
    *cfg = next;
    Ok(cfg.color_favorites.clone())
}
#[tauri::command]
pub fn remove_color_favorite(
    name: String,
    state: tauri::State<AppState>,
) -> Result<Vec<ColorFavorite>, String> {
    let mut cfg = state.config.lock().unwrap();
    let mut next = cfg.clone();
    color_favorite::remove(&mut next.color_favorites, &name)?;
    config_io::save_config(&next)?;
    *cfg = next;
    Ok(cfg.color_favorites.clone())
}
