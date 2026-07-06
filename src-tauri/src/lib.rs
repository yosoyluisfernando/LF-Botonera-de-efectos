pub mod app_setup;
pub mod domain;
pub mod engine;
#[macro_use]
pub mod ipc;
pub mod model;

use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<model::AppConfig>>,
    pub audio: Mutex<engine::audio::AudioEngine>,
    pub history: Mutex<engine::persist::history::ConfigHistory>,
    pub random_folders: Mutex<domain::button::random_folder::RandomFolderState>,
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
            random_folders: Mutex::new(domain::button::random_folder::RandomFolderState::default()),
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
