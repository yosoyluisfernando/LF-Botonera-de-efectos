//! Modulo: engine/player/resolve.rs
//! Proposito: resolver una entrada de la cola EN EL MOMENTO de sonar. Los tipos
//! especiales no se pueden resolver antes: la hora avanza, el clima cambia y una
//! carpeta aleatoria debe dar una cancion distinta en cada pasada.
//!
//! No inventa nada: reutiliza `RandomFolderState` (bolsa mezclada por fila),
//! `resolve_time_files` / `resolve_climate_file` (locuciones) y `resolve_edit`
//! (cue y ganancia del editor). Recibe solo los `Arc` que necesita, nunca el
//! `AppState`: ese contiene el propio motor y se formaria un ciclo.
use super::queue::QueueEntry;
use crate::domain::button::random_folder::RandomFolderState;
use crate::domain::playback::edit::resolve_edit;
use crate::engine::audio::formats::probe_duration_secs;
use crate::engine::persist::tracks::TrackStore;
use crate::engine::weather::client as weather;
use crate::engine::weather::playback as locution_playback;
use crate::engine::weather::resolver as locutions;
use crate::model::AppConfig;
use std::sync::{Arc, Mutex};

/// Lo que hay que sonar, ya resuelto. Varios `paths` = locucion en secuencia
/// (p. ej. "son" + "las" + "tres"), que suena como una sola pista.
pub struct ResolvedPlayback {
    pub paths: Vec<String>,
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    pub gain: f32,
    pub duration_s: f64,
}

pub struct QueueResolver {
    config: Arc<Mutex<AppConfig>>,
    random_folders: Arc<Mutex<RandomFolderState>>,
    tracks: Arc<Mutex<TrackStore>>,
}

impl QueueResolver {
    pub fn new(
        config: Arc<Mutex<AppConfig>>,
        random_folders: Arc<Mutex<RandomFolderState>>,
        tracks: Arc<Mutex<TrackStore>>,
    ) -> Self {
        Self { config, random_folders, tracks }
    }

    /// `None` = no se pudo resolver (carpeta vacia, sin clima, falta el archivo
    /// de esa hora). El motor lo salta y sigue con la siguiente: la musica de
    /// fondo no debe morirse porque falte una locucion.
    pub fn resolve(&self, entry: &QueueEntry) -> Option<ResolvedPlayback> {
        match entry.kind.as_str() {
            "audio" | "" => self.single(entry.path.clone(), entry.duration_s),
            "random_folder" => self.random(entry),
            "time" => self.time(entry),
            "temperature" | "humidity" => self.climate(entry),
            _ => None,
        }
    }

    /// Una pista suelta, con el cue y la ganancia que dejo el editor.
    fn single(&self, path: String, fallback_dur: f64) -> Option<ResolvedPlayback> {
        if path.is_empty() {
            return None;
        }
        let edit = resolve_edit(&self.tracks, &path, fallback_dur);
        Some(ResolvedPlayback {
            paths: vec![path],
            cue_start_s: edit.cue_start_s,
            cue_end_s: edit.cue_end_s,
            gain: edit.file_gain,
            duration_s: edit.duration,
        })
    }

    /// Carpeta aleatoria: la bolsa por fila ya evita repetir hasta agotarla, asi
    /// que cada pasada suena una cancion distinta.
    fn random(&self, entry: &QueueEntry) -> Option<ResolvedPlayback> {
        let path = self
            .random_folders
            .lock()
            .unwrap()
            .active_or_next_audio(&entry.id, &entry.folder, false)
            .ok()?;
        let dur = probe_duration_secs(&path);
        self.single(path, dur.max(0.0))
    }

    /// Locucion horaria: varios archivos que suenan seguidos como una pista.
    fn time(&self, entry: &QueueEntry) -> Option<ResolvedPlayback> {
        let folder = {
            let cfg = self.config.lock().unwrap();
            locution_playback::resolve_time_folder(&cfg, self.row_folder(entry)).ok()?
        };
        let paths = locutions::resolve_time_files(&folder).ok()?;
        Self::sequence(paths)
    }

    /// Locucion de temperatura o humedad segun el clima de ahora mismo.
    fn climate(&self, entry: &QueueEntry) -> Option<ResolvedPlayback> {
        let folder = {
            let cfg = self.config.lock().unwrap();
            locution_playback::resolve_climate_folder(&cfg, &entry.kind, self.row_folder(entry))
                .ok()?
        };
        let now = weather::weather_now(&self.config, false).ok()?;
        let value = if entry.kind == "humidity" { now.hum } else { now.temp };
        let file = locutions::resolve_climate_file(&folder, &entry.kind, value).ok()?;
        Self::sequence(vec![file])
    }

    /// La carpeta propia de la fila, si la trae. `None` = que decida la
    /// configuracion, igual que en un boton: la resolucion es la MISMA
    /// (`resolve_time_folder` / `resolve_climate_folder`), no una copia.
    ///
    /// Las listas del LFA no traen carpeta: llevan un marcador (`time_locution`)
    /// y cada aplicacion resuelve con SUS carpetas. El adaptador `.LFPlay` ya
    /// convierte ese marcador en carpeta vacia.
    fn row_folder<'a>(&self, entry: &'a QueueEntry) -> Option<&'a str> {
        Some(entry.folder.as_str()).filter(|f| !f.trim().is_empty())
    }

    /// Locucion: sin cue ni ganancia del editor (son archivos del sistema, no
    /// pistas que el operador edite). La duracion es la suma de sus partes.
    fn sequence(paths: Vec<String>) -> Option<ResolvedPlayback> {
        if paths.is_empty() {
            return None;
        }
        let duration_s = paths
            .iter()
            .map(|p| probe_duration_secs(p))
            .filter(|d| *d > 0.0)
            .sum();
        Some(ResolvedPlayback { paths, cue_start_s: 0.0, cue_end_s: None, gain: 1.0, duration_s })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver_with(cfg: AppConfig) -> QueueResolver {
        QueueResolver {
            config: Arc::new(Mutex::new(cfg)),
            random_folders: Arc::new(Mutex::new(RandomFolderState::default())),
            tracks: Arc::new(Mutex::new(TrackStore::open())),
        }
    }

    fn entry(kind: &str, folder: &str) -> QueueEntry {
        QueueEntry { id: "x".into(), kind: kind.into(), folder: folder.into(), ..Default::default() }
    }

    /// Sin carpeta en la fila NI en ajustes no hay nada que sonar: se salta.
    #[test]
    fn locution_without_any_folder_is_skipped() {
        let r = resolver_with(AppConfig::default());
        assert!(r.resolve(&entry("time", "")).is_none());
    }

    /// La carpeta propia de la fila se respeta; sin ella, decide la config.
    /// La resolucion es la MISMA que la de los botones, no una copia.
    #[test]
    fn the_row_folder_is_passed_through_when_it_has_one() {
        let r = resolver_with(AppConfig::default());
        assert_eq!(r.row_folder(&entry("time", "C:/propia")), Some("C:/propia"));
    }

    #[test]
    fn no_row_folder_lets_the_config_decide() {
        let r = resolver_with(AppConfig::default());
        assert_eq!(r.row_folder(&entry("time", "")), None);
        assert_eq!(r.row_folder(&entry("time", "   ")), None, "espacios no son carpeta");
    }

    /// El modulo de locuciones apagado: no suena, aunque haya carpeta en Ajustes.
    /// Esa regla ya vivia en `resolve_time_folder` y ahora se comparte.
    #[test]
    fn a_locution_is_skipped_when_the_module_is_off() {
        let mut cfg = AppConfig::default();
        cfg.weather_module_enabled = false;
        cfg.locutions.time_enabled = true;
        cfg.locutions.time_folder = "C:/hora".into();
        let r = resolver_with(cfg);
        assert!(r.resolve(&entry("time", "")).is_none());
    }

    #[test]
    fn an_empty_sequence_is_not_playable() {
        assert!(QueueResolver::sequence(Vec::new()).is_none());
    }

    /// Un tipo desconocido no revienta: simplemente no suena.
    #[test]
    fn unknown_kind_is_skipped() {
        let r = resolver_with(AppConfig::default());
        assert!(r.resolve(&entry("stream", "http://x")).is_none());
    }
}
