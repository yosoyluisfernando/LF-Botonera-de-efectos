//! Modulo: engine/player/queue.rs
//! Proposito: datos y estado de la cola del reproductor (sin audio). Las
//! operaciones de avance/pre-carga viven en `queue_ops.rs`. Testeable sin
//! dispositivo. Aplica la regla pura de `domain::player`.
use crate::domain::player::PlayerMode;

/// Una pista de la cola. Los tipos especiales (carpeta aleatoria, hora, clima)
/// NO se resuelven aqui: la hora y el clima cambian, y el aleatorio debe dar una
/// cancion distinta en cada pasada. Por eso se guardan `kind` y `folder`, y la
/// resolucion ocurre al cargar en el deck (ver `engine/player/resolve.rs`).
#[derive(Clone, Default, Debug)]
pub struct QueueEntry {
    /// Identidad estable de la fila. No depende de su posicion en la cola.
    pub id: String,
    /// "audio" | "random_folder" | "time" | "temperature" | "humidity".
    pub kind: String,
    /// Ruta del archivo. Solo para "audio"; los demas usan `folder`.
    pub path: String,
    /// Carpeta de origen. Solo para los tipos especiales.
    pub folder: String,
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    pub gain: f32,
    pub duration_s: f64,
}

impl QueueEntry {
    /// Los especiales se resuelven al sonar: basta con tener carpeta.
    pub fn is_playable(&self) -> bool {
        if self.kind == "audio" || self.kind.is_empty() {
            !self.path.is_empty()
        } else {
            !self.folder.is_empty()
        }
    }

    /// La hora y el clima dependen de CUANDO suenan, y la pre-carga ocurre
    /// mientras suena la pista anterior: precargarlos diria la hora de hace
    /// varios minutos. Estos se resuelven en el relevo, no antes. El aleatorio
    /// si se precarga: elegir la cancion por adelantado no la estropea.
    pub fn needs_late_resolve(&self) -> bool {
        matches!(self.kind.as_str(), "time" | "temperature" | "humidity")
    }
}

/// Accion sobre un deck (0 o 1). El hilo la traduce a operaciones de rodio.
#[derive(Debug)]
pub enum DeckAction {
    Load { deck: usize, entry: QueueEntry, autoplay: bool },
    /// Saltar a una posicion de la pista ya cargada, sin cambiar de pista.
    Seek { deck: usize, position_s: f64 },
    Resume { deck: usize },
    /// Detiene un solo deck (el saliente en un relevo forzado, sin cortar el que
    /// acaba de arrancar). Evita que dos pistas suenen a la vez (solapamiento).
    Stop { deck: usize },
    StopAll,
}

pub struct QueueState {
    pub(super) entries: Vec<QueueEntry>,
    pub(super) mode: PlayerMode,
    pub(super) stop_after: bool,
    /// Boton Loop: al acabar, la pista actual vuelve a empezar en vez de avanzar.
    /// Es de TRANSPORTE, como `stop_after`: sigue puesto al cambiar de cancion y
    /// no se persiste. Ojo, no confundir con el modo `Repeat`, que repite la
    /// LISTA entera; este repite UNA cancion.
    pub(super) loop_current: bool,
    pub(super) current: Option<usize>,
    pub(super) cursor: Option<usize>,
    pub(super) marked: Option<usize>,
    pub(super) active_deck: usize,
    pub(super) loaded_other: Option<usize>,
    pub(super) rng: u64,
}

impl QueueState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            mode: PlayerMode::Normal,
            stop_after: false,
            loop_current: false,
            current: None,
            cursor: None,
            marked: None,
            active_deck: 0,
            loaded_other: None,
            rng: 0x2545_F491_4F6C_DD1D,
        }
    }

    pub fn active_deck(&self) -> usize {
        self.active_deck
    }
    pub fn current(&self) -> Option<usize> {
        self.current
    }
    /// Indice que la UI pinta en naranja: lo marcado o lo ya precargado.
    pub fn next(&self) -> Option<usize> {
        self.marked.or(self.loaded_other)
    }
    pub fn mode(&self) -> PlayerMode {
        self.mode
    }
    pub fn stop_after(&self) -> bool {
        self.stop_after
    }
    pub fn loop_current(&self) -> bool {
        self.loop_current
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn playable(&self, i: usize) -> bool {
        self.entries.get(i).is_some_and(QueueEntry::is_playable)
    }

    pub(super) fn index_of(&self, id: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.id == id)
    }

    pub(super) fn rand(&mut self) -> f64 {
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng >> 33) as f64) / ((1u64 << 31) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn special(kind: &str, folder: &str) -> QueueEntry {
        QueueEntry { id: "x".into(), kind: kind.into(), folder: folder.into(), ..Default::default() }
    }

    #[test]
    fn audio_needs_a_file() {
        let with = QueueEntry { kind: "audio".into(), path: "C:/a.mp3".into(), ..Default::default() };
        assert!(with.is_playable());
        assert!(!QueueEntry { kind: "audio".into(), ..Default::default() }.is_playable());
    }

    /// Los especiales solo necesitan carpeta: el archivo se elige al sonar.
    #[test]
    fn special_types_only_need_a_folder() {
        for kind in ["random_folder", "time", "temperature", "humidity"] {
            assert!(special(kind, "C:/carpeta").is_playable(), "{kind} deberia sonar");
            assert!(!special(kind, "").is_playable(), "{kind} sin carpeta no");
        }
    }

    /// La hora y el clima dependen de cuando suenan: no se pueden precargar
    /// minutos antes. El aleatorio si.
    #[test]
    fn only_time_and_weather_resolve_late() {
        for kind in ["time", "temperature", "humidity"] {
            assert!(special(kind, "C:/x").needs_late_resolve(), "{kind} depende de la hora");
        }
        assert!(!special("random_folder", "C:/x").needs_late_resolve());
        let audio = QueueEntry { kind: "audio".into(), path: "a.mp3".into(), ..Default::default() };
        assert!(!audio.needs_late_resolve());
    }

    /// Antes se saltaban por no tener ruta; ahora la cola los tiene en cuenta.
    #[test]
    fn the_queue_no_longer_skips_special_types() {
        let mut q = QueueState::new();
        q.entries = vec![
            QueueEntry { kind: "audio".into(), path: "C:/a.mp3".into(), ..Default::default() },
            special("random_folder", "C:/musica"),
        ];
        assert!(q.playable(0));
        assert!(q.playable(1), "la carpeta aleatoria cuenta como reproducible");
    }
}
