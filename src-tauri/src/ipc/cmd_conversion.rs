//! Consulta del estado actual para hidratar la UI tras abrir o redibujar.
use crate::engine::audio::transcode::progress::{self, ConversionSnapshot};
#[tauri::command]
pub fn get_audio_conversions() -> Vec<ConversionSnapshot> { progress::snapshots() }
