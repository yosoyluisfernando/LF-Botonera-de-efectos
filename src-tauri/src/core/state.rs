use crate::domain::button::random_folder::RandomFolderState;
use crate::domain::library::protected_tracks;
use crate::engine::audio::AudioEngine;
use crate::engine::cache::track_analysis::TrackAnalysisCache;
use crate::engine::console::ConsoleEngine;
use crate::engine::dsp::waveform::WaveformCache;
use crate::engine::input::midi::MidiEngine;
use crate::engine::library::service::LibraryService;
use crate::engine::persist::config_io;
use crate::engine::persist::history::ConfigHistory;
use crate::engine::persist::last_played::LastPlayed;
use crate::engine::persist::tracks::TrackStore;
use crate::engine::player::{PlayerEngine, QueueResolver};
use crate::model::AppConfig;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    /// La consola: dueña de las salidas fisicas y de los buses. Arc porque los
    /// motores de audio son sus clientes y la comparten.
    pub console: Arc<ConsoleEngine>,
    pub audio: Mutex<AudioEngine>,
    pub player: Mutex<PlayerEngine>,
    pub midi: MidiEngine,
    pub history: Mutex<ConfigHistory>,
    /// Arc porque lo comparte el resolvedor del reproductor: las bolsas de
    /// aleatorios son las mismas para los botones y para la cola.
    pub random_folders: Arc<Mutex<RandomFolderState>>,
    pub tracks: Arc<Mutex<TrackStore>>,
    pub library: Arc<LibraryService>,
    pub waveforms: Mutex<WaveformCache>,
    pub track_analysis: Mutex<TrackAnalysisCache>,
    pub last_played: LastPlayed,
    /// Serializa la creación y preparación de respaldos. SQLite sigue
    /// atendiendo audio/Biblioteca mediante su copia en línea.
    pub backup_operation: Mutex<()>,
}

impl AppState {
    pub fn new() -> Self {
        // La consola va primero: es la dueña de las salidas y de los buses, y el
        // motor de efectos le pide los suyos nada mas nacer.
        let console = Arc::new(ConsoleEngine::new());
        // El reproductor auxiliar comparte la cache de precarga de los efectos
        // (PCM en RAM) para no duplicar memoria. Se crea el motor de efectos
        // primero para tomar ese handle.
        let audio = AudioEngine::new(Arc::clone(&console));
        // Lo que el reproductor necesita para resolver los tipos especiales al
        // sonar. Se crea antes que el motor para pasarselo ya montado: darle el
        // AppState entero formaria un ciclo, porque contiene el propio motor.
        let config = Arc::new(Mutex::new(config_io::load_config()));
        let random_folders = Arc::new(Mutex::new(RandomFolderState::default()));
        let tracks = Arc::new(Mutex::new(TrackStore::open()));
        let library = Arc::new(LibraryService::open_default());
        let resolver = QueueResolver::new(
            Arc::clone(&config),
            Arc::clone(&random_folders),
            Arc::clone(&tracks),
        );
        let player =
            PlayerEngine::new(audio.preload_cache_handle(), resolver, Arc::clone(&console));
        let cleanup_library = Arc::clone(&library);
        let cleanup_config = Arc::clone(&config);
        let _ = std::thread::Builder::new()
            .name("library-retention".into())
            .spawn(move || {
                let Ok(config_guard) = cleanup_config.lock() else {
                    return;
                };
                let protected = protected_tracks::from_config(&config_guard);
                if let Err(error) = cleanup_library.purge_expired(&protected) {
                    eprintln!("library retention cleanup unavailable: {error}");
                }
            });
        Self {
            config,
            console,
            audio: Mutex::new(audio),
            player: Mutex::new(player),
            midi: MidiEngine::new(),
            history: Mutex::new(ConfigHistory::default()),
            random_folders,
            tracks,
            library,
            waveforms: Mutex::new(WaveformCache::default()),
            track_analysis: Mutex::new(TrackAnalysisCache::default()),
            last_played: LastPlayed::new(),
            backup_operation: Mutex::new(()),
        }
    }
}
