use crate::domain::button::random_folder::RandomFolderState;
use crate::engine::audio::AudioEngine;
use crate::engine::cache::track_analysis::TrackAnalysisCache;
use crate::engine::dsp::waveform::WaveformCache;
use crate::engine::persist::config_io;
use crate::engine::persist::history::ConfigHistory;
use crate::engine::persist::last_played::LastPlayed;
use crate::engine::persist::tracks::TrackStore;
use crate::model::AppConfig;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub audio: Mutex<AudioEngine>,
    pub history: Mutex<ConfigHistory>,
    pub random_folders: Mutex<RandomFolderState>,
    pub tracks: Arc<Mutex<TrackStore>>,
    pub waveforms: Mutex<WaveformCache>,
    pub track_analysis: Mutex<TrackAnalysisCache>,
    pub last_played: LastPlayed,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(config_io::load_config())),
            audio: Mutex::new(AudioEngine::new()),
            history: Mutex::new(ConfigHistory::default()),
            random_folders: Mutex::new(RandomFolderState::default()),
            tracks: Arc::new(Mutex::new(TrackStore::open())),
            waveforms: Mutex::new(WaveformCache::default()),
            track_analysis: Mutex::new(TrackAnalysisCache::default()),
            last_played: LastPlayed::new(),
        }
    }
}
