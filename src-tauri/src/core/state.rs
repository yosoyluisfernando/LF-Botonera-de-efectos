use crate::domain::button::random_folder::RandomFolderState;
use crate::engine::audio::AudioEngine;
use crate::engine::cache::track_analysis::TrackAnalysisCache;
use crate::engine::player::{PlayerEngine, QueueResolver};
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
    pub player: Mutex<PlayerEngine>,
    pub history: Mutex<ConfigHistory>,
    /// Arc porque lo comparte el resolvedor del reproductor: las bolsas de
    /// aleatorios son las mismas para los botones y para la cola.
    pub random_folders: Arc<Mutex<RandomFolderState>>,
    pub tracks: Arc<Mutex<TrackStore>>,
    pub waveforms: Mutex<WaveformCache>,
    pub track_analysis: Mutex<TrackAnalysisCache>,
    pub last_played: LastPlayed,
}

impl AppState {
    pub fn new() -> Self {
        // El reproductor auxiliar comparte la cache de precarga de los efectos
        // (PCM en RAM) para no duplicar memoria. Se crea el motor de efectos
        // primero para tomar ese handle.
        let audio = AudioEngine::new();
        // Lo que el reproductor necesita para resolver los tipos especiales al
        // sonar. Se crea antes que el motor para pasarselo ya montado: darle el
        // AppState entero formaria un ciclo, porque contiene el propio motor.
        let config = Arc::new(Mutex::new(config_io::load_config()));
        let random_folders = Arc::new(Mutex::new(RandomFolderState::default()));
        let tracks = Arc::new(Mutex::new(TrackStore::open()));
        let resolver = QueueResolver::new(
            Arc::clone(&config),
            Arc::clone(&random_folders),
            Arc::clone(&tracks),
        );
        let player = PlayerEngine::new(audio.preload_cache_handle(), resolver);
        Self {
            config,
            audio: Mutex::new(audio),
            player: Mutex::new(player),
            history: Mutex::new(ConfigHistory::default()),
            random_folders,
            tracks,
            waveforms: Mutex::new(WaveformCache::default()),
            track_analysis: Mutex::new(TrackAnalysisCache::default()),
            last_played: LastPlayed::new(),
        }
    }
}
