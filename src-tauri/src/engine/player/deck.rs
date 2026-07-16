//! Modulo: engine/player/deck.rs
//! Proposito: un "deck" del reproductor auxiliar. Envuelve un Sink de rodio y
//! lleva el estado de la pista cargada. Reproduce una pista a la vez; el motor
//! usa dos decks para la pre-carga ping-pong (Fase B). Patron adaptado de
//! player.rs de LF Automatizador 2.0. No hace I/O de red ni decide el avance.
use crate::engine::audio::decode::BoxSource;
use rodio::{OutputStreamHandle, Sink, Source};
use std::time::Duration;

/// Estado de un deck. `Loaded` = pre-cargado en pausa, listo para arrancar al
/// instante. `Finished` = termino de forma natural (sink vacio) y espera relevo.
/// `Failed` = no se pudo cargar (carpeta vacia, sin clima, archivo ilegible); se
/// trata como "termino" para que el motor releve y la musica siga.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeckStatus {
    Empty,
    Loaded,
    Playing,
    Paused,
    Finished,
    Failed,
}

/// Envuelve una `BoxSource` para poder anexarla a un `Sink`: `Box<dyn Source>`
/// no implementa `Source` por si mismo, asi que delegamos cada metodo.
struct DeckSource {
    inner: BoxSource,
}
impl Iterator for DeckSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        self.inner.next()
    }
}
impl Source for DeckSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

/// Lo que el deck recuerda de la pista cargada. Hace falta para poder
/// RECONSTRUIRLA en otra posicion: rodio no sabe reposicionar un `Sink`, asi que
/// un seek es volver a crear la fuente desde el punto pedido.
#[derive(Clone, Default)]
pub struct DeckTrack {
    /// Ruta YA resuelta (la que suena). Para una carpeta aleatoria es la cancion
    /// elegida, no la carpeta: al recargar debe sonar la misma, no otra.
    pub path: String,
    pub duration_s: f64,
    pub gain: f32,
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    /// Una locucion son varios archivos encadenados en una sola fuente: no se
    /// puede reposicionar. La barra de progreso lo respeta y no deja arrastrar.
    pub seekable: bool,
}

pub struct Deck {
    sink: Sink,
    status: DeckStatus,
    track: DeckTrack,
    /// Segundos que ya habian pasado al recargar por un seek: `sink.get_pos()`
    /// vuelve a cero con la fuente nueva, asi que sin esto el tiempo retrocederia.
    position_offset_s: f64,
}

impl Deck {
    /// Crea un deck en pausa sobre el dispositivo dado. None si el Sink no se crea.
    pub fn new(handle: &OutputStreamHandle) -> Option<Self> {
        let sink = Sink::try_new(handle).ok()?;
        sink.pause();
        Some(Self {
            sink,
            status: DeckStatus::Empty,
            track: DeckTrack::default(),
            position_offset_s: 0.0,
        })
    }

    /// Pre-carga una fuente (con cue/cache ya aplicados) y la deja lista en pausa.
    pub fn load(&mut self, source: BoxSource, track: DeckTrack, volume: f32) {
        self.sink.stop();
        self.sink.append(DeckSource { inner: source });
        self.sink.set_volume((volume * track.gain).max(0.0));
        self.sink.pause();
        self.track = track;
        self.position_offset_s = 0.0;
        self.status = DeckStatus::Loaded;
    }

    /// Recarga la MISMA pista desde `offset_s` (seek). Conserva si sonaba o
    /// estaba en pausa: saltar de posicion no debe arrancar ni parar nada.
    pub fn reload_at(&mut self, source: BoxSource, offset_s: f64, volume: f32) {
        let was_playing = self.status == DeckStatus::Playing;
        self.sink.stop();
        self.sink.append(DeckSource { inner: source });
        self.sink.set_volume((volume * self.track.gain).max(0.0));
        self.position_offset_s = offset_s;
        if was_playing {
            self.sink.play();
            self.status = DeckStatus::Playing;
        } else {
            self.sink.pause();
            self.status = DeckStatus::Paused;
        }
    }

    /// Datos de la pista cargada, para poder reconstruirla en un seek.
    pub fn track(&self) -> &DeckTrack {
        &self.track
    }

    pub fn play(&mut self) {
        if matches!(self.status, DeckStatus::Loaded | DeckStatus::Paused) {
            self.sink.play();
            self.status = DeckStatus::Playing;
        }
    }

    pub fn pause(&mut self) {
        if self.status == DeckStatus::Playing {
            self.sink.pause();
            self.status = DeckStatus::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.status = DeckStatus::Empty;
        self.track = DeckTrack::default();
        self.position_offset_s = 0.0;
    }

    /// Aplica el volumen del motor multiplicando por la ganancia de la pista.
    pub fn apply_volume(&self, volume: f32) {
        self.sink.set_volume((volume * self.track.gain).max(0.0));
    }

    /// Marca `Finished` una sola vez, en la transicion, si el sink se vacio
    /// mientras reproducia. El motor lo usara para el relevo ping-pong (Fase B).
    pub fn poll_finished(&mut self) -> bool {
        // `Failed` cuenta como terminado: si una pista no se pudo resolver, el
        // motor debe relevarla en vez de quedarse callado esperandola.
        if (self.status == DeckStatus::Playing && self.sink.empty())
            || self.status == DeckStatus::Failed
        {
            self.status = DeckStatus::Finished;
            return true;
        }
        false
    }

    /// La pista no se pudo cargar. Deja el deck listo para que lo releven.
    pub fn fail(&mut self) {
        self.sink.stop();
        self.track = DeckTrack::default();
        self.position_offset_s = 0.0;
        self.status = DeckStatus::Failed;
    }

    pub fn status(&self) -> DeckStatus {
        self.status
    }
    pub fn path(&self) -> Option<&str> {
        Some(self.track.path.as_str()).filter(|p| !p.is_empty())
    }
    pub fn duration_s(&self) -> f64 {
        self.track.duration_s
    }
    /// Se suma el offset del ultimo seek: la fuente nueva cuenta desde cero.
    pub fn position_s(&self) -> f64 {
        self.position_offset_s + self.sink.get_pos().as_secs_f64()
    }
    /// Solo se puede saltar de posicion en una pista con duracion conocida y de
    /// un solo archivo (una locucion son varios encadenados).
    pub fn can_seek(&self) -> bool {
        self.track.seekable && self.track.duration_s > 0.0 && !self.track.path.is_empty()
    }
}
