pub mod app_setup;
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
pub mod engine;
pub mod export_tracks;
pub mod grid_move;
pub mod grid_reorder;
pub mod grid_resize;
pub mod grid_view;
pub mod lfa_format;
pub mod model;
pub mod playback_mode;
pub mod playback_seek;
pub mod playback_source;
pub mod playback_state;
pub mod random_folder;

#[macro_use]
mod register_handlers;

use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<model::AppConfig>>,
    pub audio: Mutex<engine::audio::AudioEngine>,
    pub history: Mutex<engine::persist::history::ConfigHistory>,
    pub random_folders: Mutex<random_folder::RandomFolderState>,
    pub tracks: Arc<Mutex<engine::persist::tracks::TrackStore>>,
    pub waveforms: Mutex<engine::dsp::waveform::WaveformCache>,
    pub track_analysis: Mutex<engine::cache::track_analysis::TrackAnalysisCache>,
    pub last_played: engine::persist::last_played::LastPlayed,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            config: Arc::new(Mutex::new(engine::persist::config_io::load_config())),
            audio: Mutex::new(engine::audio::AudioEngine::new()),
            history: Mutex::new(engine::persist::history::ConfigHistory::default()),
            random_folders: Mutex::new(random_folder::RandomFolderState::default()),
            tracks: Arc::new(Mutex::new(engine::persist::tracks::TrackStore::open())),
            waveforms: Mutex::new(engine::dsp::waveform::WaveformCache::default()),
            track_analysis: Mutex::new(engine::cache::track_analysis::TrackAnalysisCache::default()),
            last_played: engine::persist::last_played::LastPlayed::new(),
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
        .plugin(engine::input::keyboard::plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(lf_invoke_handlers!())
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicacion Tauri");
}
