/// Módulo: lib.rs
/// Propósito: Define AppState, declara módulos y conecta todo en `run()`.
/// No contiene lógica de negocio: eso va en cmd_*.rs, config.rs y audio.rs.
pub mod app_setup;
pub mod audio;
pub mod audio_analysis;
pub mod audio_command;
pub mod audio_decode;
pub mod audio_device;
pub mod audio_formats;
pub mod audio_monitor;
pub mod audio_thread;
pub mod button_defaults;
pub mod button_types;
pub mod cached_source;
pub mod cmd_audio;
pub mod cmd_button_flags;
pub mod cmd_button_playback;
pub mod cmd_button_types;
pub mod cmd_button_update;
pub mod cmd_export;
pub mod cmd_grid;
pub mod cmd_history;
pub mod cmd_keys;
pub mod cmd_local_shortcuts;
pub mod cmd_locutions;
pub mod cmd_master_volume;
pub mod cmd_meta;
pub mod cmd_paletas;
pub mod cmd_playback;
pub mod cmd_preload;
pub mod cmd_profiles;
pub mod cmd_tracks;
pub mod cmd_updates;
pub mod colors;
pub mod config;
pub mod config_history;
pub mod cue_source;
pub mod db;
pub mod geocode;
pub mod global_shortcuts;
pub mod last_played;
pub mod grid_move;
pub mod grid_reorder;
pub mod grid_resize;
pub mod grid_view;
pub mod lfa_format;
pub mod locution_playback;
pub mod locutions;
pub mod master_bus;
pub mod master_button;
pub mod playback_mode;
pub mod playback_state;
pub mod preload_cache;
pub mod preload_warm;
pub mod preloader;
pub mod random_folder;
pub mod shortcut_rules;
pub mod tab_reorder;
pub mod track_store;
pub mod types;
pub mod types_audio;
pub mod types_grid;
pub mod types_locutions;
pub mod types_preload;
pub mod types_track;
pub mod vu_meter;
pub mod waveform;
pub mod weather;

use std::sync::{Arc, Mutex};

// ─── Estado global ────────────────────────────────────────────────────────────

pub struct AppState {
    /// Arc permite compartir el config con hilos de fondo (reloj, monitor).
    pub config: Arc<Mutex<types::AppConfig>>,
    pub audio: Mutex<audio::AudioEngine>,
    pub history: Mutex<config_history::ConfigHistory>,
    pub random_folders: Mutex<random_folder::RandomFolderState>,
    /// Metadatos por archivo del editor de pistas (cue, dB, LUFS) en tracks.db.
    /// Arc para compartir con el hilo que vuelca el historial de reproducción.
    pub tracks: Arc<Mutex<track_store::TrackStore>>,
    /// Envolventes de onda en memoria mientras se edita (zoom); no se persisten.
    pub waveforms: Mutex<waveform::WaveformCache>,
    /// Historial de última reproducción en memoria (debounce a tracks.db).
    pub last_played: last_played::LastPlayed,
}

// ─── Arranque ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            config: Arc::new(Mutex::new(config::load_config())),
            audio: Mutex::new(audio::AudioEngine::new()),
            history: Mutex::new(config_history::ConfigHistory::default()),
            random_folders: Mutex::new(random_folder::RandomFolderState::default()),
            tracks: Arc::new(Mutex::new(track_store::TrackStore::open())),
            waveforms: Mutex::new(waveform::WaveformCache::default()),
            last_played: last_played::LastPlayed::new(),
        })
        .setup(app_setup::on_setup)
        .plugin(global_shortcuts::plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Config / primer arranque
            cmd_profiles::get_config,
            cmd_profiles::set_first_boot_complete,
            cmd_profiles::set_theme,
            cmd_profiles::set_language,
            cmd_profiles::set_button_text_size,
            // Perfiles
            cmd_profiles::set_active_profile,
            cmd_profiles::create_profile,
            cmd_profiles::delete_profile,
            cmd_profiles::update_profile_meta,
            // Pestañas
            cmd_paletas::set_active_paleta,
            cmd_paletas::create_paleta,
            cmd_paletas::delete_paleta,
            cmd_paletas::update_paleta_meta,
            tab_reorder::reorder_paletas,
            // Audio
            cmd_audio::get_audio_devices,
            cmd_audio::set_audio_device,
            cmd_audio::play_audio,
            cmd_audio::stop_audio,
            cmd_audio::stop_all_audio,
            cmd_button_playback::play_button,
            // Grid / botones
            cmd_grid::get_grid_state,
            cmd_grid::suggest_button_style,
            cmd_grid::get_color_palette,
            cmd_grid::assign_file_to_button,
            cmd_grid::clear_button,
            cmd_history::undo_config,
            cmd_history::redo_config,
            cmd_button_flags::toggle_button_flag,
            cmd_button_types::get_edit_button_types,
            cmd_button_update::update_button_data,
            grid_move::move_button_to_paleta,
            grid_reorder::reorder_buttons,
            // Audio util
            cmd_audio::set_audio_volume,
            cmd_master_volume::get_master_volume_state,
            cmd_master_volume::set_master_volume,
            cmd_master_volume::set_master_volume_options,
            // Atajos globales
            cmd_keys::set_global_keys,
            cmd_keys::cycle_paleta,
            cmd_local_shortcuts::handle_local_shortcut,
            cmd_keys::clear_button_shortcut,
            // Locuciones dinámicas (Fase 6)
            cmd_locutions::set_locution_config,
            cmd_locutions::pick_named_folder,
            cmd_locutions::search_city,
            cmd_locutions::preview_weather,
            cmd_locutions::get_weather_now,
            cmd_locutions::play_time_locution,
            cmd_locutions::play_climate_locution,
            // Export / Import
            cmd_export::export_tab,
            cmd_export::export_tab_by_id,
            cmd_export::import_tab,
            cmd_export::export_profile,
            cmd_export::export_profile_by_id,
            cmd_export::import_profile,
            // Metadatos de la aplicación
            cmd_meta::get_app_version,
            cmd_meta::toggle_clock_format,
            // Actualizaciones
            cmd_updates::check_for_updates,
            // Modo de reproducción global (Fase 7.5)
            cmd_playback::get_playback_mode,
            cmd_playback::get_playback_state,
            cmd_playback::set_playback_mode,
            cmd_playback::set_solo_mode,
            // Editor de pistas (cue + normalizador + onda)
            cmd_tracks::analyze_track,
            cmd_tracks::waveform_view,
            cmd_tracks::get_track_meta,
            cmd_tracks::set_track_cue,
            cmd_tracks::set_track_gain,
            cmd_tracks::set_track_normalization,
            // Precarga de audio (configuración; caché en etapas posteriores)
            cmd_preload::get_preload_config,
            cmd_preload::should_prompt_preload,
            cmd_preload::mark_preload_prompted,
            cmd_preload::set_preload_config,
            cmd_preload::get_preload_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicación Tauri");
}
