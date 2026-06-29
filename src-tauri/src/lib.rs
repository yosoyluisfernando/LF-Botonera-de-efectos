pub mod app_setup;
pub mod audio;
pub mod audio_analysis;
pub mod audio_command;
pub mod audio_decode;
pub mod audio_device;
pub mod audio_device_list;
pub mod audio_formats;
pub mod audio_monitor;
pub mod audio_ops;
pub mod audio_thread;
pub mod audio_thread_play;
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
pub mod cmd_playback_progress;
pub mod cmd_preload;
pub mod cmd_profiles;
pub mod cmd_startup_prompts;
pub mod cmd_track_response;
pub mod cmd_tracks;
pub mod cmd_updates;
pub mod colors;
pub mod config;
pub mod config_history;
pub mod cue_detect;
pub mod cue_source;
pub mod db;
pub mod export_tracks;
pub mod fade_ramp;
pub mod geocode;
pub mod global_shortcuts;
pub mod grid_move;
pub mod grid_reorder;
pub mod grid_resize;
pub mod grid_view;
pub mod last_played;
pub mod lfa_format;
pub mod locution_playback;
pub mod locutions;
pub mod master_bus;
pub mod master_button;
pub mod playback_mode;
pub mod playback_seek;
pub mod playback_source;
pub mod playback_state;
pub mod preload_cache;
pub mod preload_warm;
pub mod preloader;
pub mod random_folder;
pub mod shortcut_rules;
pub mod tab_reorder;
pub mod track_analysis_cache;
pub mod track_store;
pub mod types;
pub mod types_audio;
pub mod types_fade;
pub mod types_grid;
pub mod types_locutions;
pub mod types_norm;
pub mod types_playback_progress;
pub mod types_preload;
pub mod types_startup;
pub mod types_track;
pub mod vu_meter;
pub mod waveform;
pub mod weather;

#[macro_use]
mod register_handlers;

use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<types::AppConfig>>,
    pub audio: Mutex<audio::AudioEngine>,
    pub history: Mutex<config_history::ConfigHistory>,
    pub random_folders: Mutex<random_folder::RandomFolderState>,
    pub tracks: Arc<Mutex<track_store::TrackStore>>,
    pub waveforms: Mutex<waveform::WaveformCache>,
    pub track_analysis: Mutex<track_analysis_cache::TrackAnalysisCache>,
    pub last_played: last_played::LastPlayed,
}

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
            track_analysis: Mutex::new(track_analysis_cache::TrackAnalysisCache::default()),
            last_played: last_played::LastPlayed::new(),
        })
        .setup(app_setup::on_setup)
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .plugin(global_shortcuts::plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(lf_invoke_handlers!())
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicacion Tauri");
}
