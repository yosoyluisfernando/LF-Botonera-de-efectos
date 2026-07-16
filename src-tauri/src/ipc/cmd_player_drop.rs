//! Modulo: cmd_player_drop.rs
//! Proposito: soltar carpetas y varios archivos a la vez en la cola del
//! reproductor. Es exclusivo del reproductor: en la botonera del panel fijo se
//! mantiene la regla de un solo archivo y nada de carpetas (son dos herramientas
//! distintas, no una incoherencia).
//!
//! **Por que en segundo plano.** Listar es barato (medido: ~21 ms para 1.910
//! archivos en 53 carpetas), pero leer la duracion cuesta ~40 ms **por archivo**:
//! esa misma carpeta serian ~75 s. En un comando normal eso congela la
//! aplicacion, asi que el trabajo va en `spawn_blocking` y se emite progreso,
//! mismo patron que el analisis del editor de pistas. Las pistas se insertan por
//! lotes, asi la lista **crece a la vista** y nunca parece que no hace nada.
use super::cmd_player_queue::{insert_track, next_id, sync_queue};
use super::cmd_player::{player_view, PlayerView};
use super::AppState;
use crate::domain::button::defaults::new_button;
use crate::engine::audio::formats::{audio_files_recursive, is_audio_path, probe_duration_secs};
use crate::engine::persist::config_io;
use crate::model::player::LARGE_FOLDER_THRESHOLD;
use crate::model::{AppConfig, ButtonData};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

/// Cada cuantas pistas se refresca la lista. Suficientemente pequeno para que se
/// vea avanzar, suficientemente grande para no inundar de eventos a la interfaz.
const BATCH: usize = 20;

/// Lo que la UI necesita saber ANTES de anadir. Quien decide si hay que
/// preguntar es Rust (regla 4): conoce el umbral y el ajuste del usuario.
#[derive(Serialize)]
pub struct DropScan {
    pub count: u32,
    /// Hay que pedir confirmacion: son muchas y el ajuste dice "preguntar".
    pub needs_confirm: bool,
    /// El usuario eligio "no anadir nunca" para carpetas grandes.
    pub blocked: bool,
}

/// Expande lo soltado: los archivos de audio pasan tal cual y las carpetas se
/// recorren enteras, subcarpetas incluidas.
fn expand(paths: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for path in paths {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            out.extend(audio_files_recursive(path));
        } else if p.is_file() && is_audio_path(p) {
            out.push(path.clone());
        }
    }
    out
}

/// Cuenta lo que se anadiria y dice si hay que preguntar. Es rapido: solo mira
/// extensiones, no abre ningun archivo.
#[tauri::command]
pub fn player_scan_drop(paths: Vec<String>, state: tauri::State<AppState>) -> DropScan {
    let count = expand(&paths).len();
    let action = state.config.lock().unwrap().player.large_folder_action.clone();
    let many = count > LARGE_FOLDER_THRESHOLD;
    DropScan {
        count: count as u32,
        needs_confirm: many && action == "ask",
        blocked: many && action == "never",
    }
}

/// Anade lo soltado. `remember` viene del check "recordar siempre" del aviso y
/// guarda la decision para no volver a preguntar; se puede cambiar en Ajustes.
#[tauri::command]
pub async fn player_add_drop(
    app: tauri::AppHandle,
    paths: Vec<String>,
    index: Option<u32>,
    remember: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerView, String> {
    if remember == Some(true) {
        let mut cfg = state.config.lock().unwrap();
        cfg.player.large_folder_action = "always".into();
        config_io::save_config(&cfg)?;
    }
    let config = Arc::clone(&state.config);
    // El trabajo pesado fuera del hilo de la interfaz: aqui esta la diferencia
    // entre una carpeta que se carga y una aplicacion congelada.
    tauri::async_runtime::spawn_blocking(move || add_all(&app, &config, paths, index))
        .await
        .map_err(|e| e.to_string())??;
    sync_queue(&state);
    Ok(player_view(&state))
}

fn add_all(
    app: &tauri::AppHandle,
    config: &Arc<Mutex<AppConfig>>,
    paths: Vec<String>,
    index: Option<u32>,
) -> Result<(), String> {
    let files = expand(&paths);
    let total = files.len();
    if total == 0 {
        return Err("player_no_audio_found".into());
    }
    // `index` es donde se suelta: las pistas se insertan una tras otra desde ahi
    // para conservar el orden del disco.
    let mut at = index;
    for (done, chunk) in files.chunks(BATCH).enumerate() {
        let mut cfg = config.lock().unwrap();
        for path in chunk {
            let btn = build_track(&cfg, path);
            insert_track(&mut cfg.player.tracks, btn, at);
            at = at.map(|i| i + 1);
        }
        drop(cfg);
        emit_progress(app, (done + 1) * chunk.len(), total);
    }
    let cfg = config.lock().unwrap();
    config_io::save_config(&cfg) // una sola escritura a disco, al terminar
}

fn build_track(cfg: &AppConfig, path: &str) -> ButtonData {
    let name = std::path::Path::new(path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    let bg = crate::domain::colors::random_color();
    let text = crate::domain::colors::text_for_theme(&bg, &cfg.theme, "button");
    let mut btn = new_button("player", 1, &name, &bg, &text);
    btn.id = next_id(&cfg.player.tracks);
    btn.duration = probe_duration_secs(path); // lo caro: ~40 ms por archivo
    btn.duration_str = if btn.duration > 0.0 {
        format!("{:.1}s", btn.duration)
    } else {
        String::new()
    };
    btn.path = path.to_string();
    btn
}

#[derive(Clone, Serialize)]
struct DropProgress {
    done: u32,
    total: u32,
}

fn emit_progress(app: &tauri::AppHandle, done: usize, total: usize) {
    let _ = app.emit(
        "player-drop-progress",
        DropProgress { done: done.min(total) as u32, total: total as u32 },
    );
}
