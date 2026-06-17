/// Módulo: lib.rs
/// Propósito: Define AppState, declara módulos y conecta todo en `run()`.
/// No contiene lógica de negocio: eso va en cmd_*.rs, config.rs y audio.rs.
pub mod audio;
pub mod audio_command;
pub mod audio_decode;
pub mod audio_device;
pub mod audio_formats;
pub mod audio_monitor;
pub mod audio_thread;
pub mod button_defaults;
pub mod button_types;
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
pub mod cmd_playback;
pub mod cmd_profiles;
pub mod cmd_updates;
pub mod colors;
pub mod config;
pub mod config_history;
pub mod global_shortcuts;
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
pub mod random_folder;
pub mod shortcut_rules;
pub mod types;
pub mod types_grid;
pub mod types_locutions;
pub mod vu_meter;
pub mod weather;

use std::sync::{Arc, Mutex};

// ─── Estado global ────────────────────────────────────────────────────────────

pub struct AppState {
    /// Arc permite compartir el config con hilos de fondo (reloj, monitor).
    pub config: Arc<Mutex<types::AppConfig>>,
    pub audio: Mutex<audio::AudioEngine>,
    pub history: Mutex<config_history::ConfigHistory>,
    pub random_folders: Mutex<random_folder::RandomFolderState>,
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
        })
        .setup(|app| {
            use tauri::Manager;
            let state = app.state::<AppState>();
            let cfg = state.config.lock().unwrap();
            let pid = cfg.active_profile_id.clone();
            let master_volume = cmd_master_volume::startup_volume(&cfg);
            let device = cfg
                .profiles
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.audio.out_main.clone())
                .unwrap_or_else(|| "default".to_string());
            drop(cfg);

            let engine = state.audio.lock().unwrap();
            let _ = engine.set_device(&device);
            engine.set_master_volume(master_volume);

            // Hilo monitor: emite "audio-tick" con progreso, tiempo restante y niveles VU
            let (ll, lr) = engine.master_levels_handles();
            audio_monitor::start(
                app.handle().clone(),
                engine.button_states_handle(),
                ll,
                lr,
                engine.last_pressed_handle(),
            );
            drop(engine);

            // Hilo del reloj: emite "clock-tick" con hora y fecha localizadas
            cmd_meta::start_clock_thread(app.handle().clone(), Arc::clone(&state.config));

            // Hilo de clima: refresca cada 15 min y emite "weather-updated"
            weather::start_auto_refresh(app.handle().clone());
            let _ = global_shortcuts::sync(app.handle());
            Ok(())
        })
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
            cmd_profiles::set_active_paleta,
            cmd_profiles::create_paleta,
            cmd_profiles::delete_paleta,
            cmd_profiles::update_paleta_meta,
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
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicación Tauri");
}
