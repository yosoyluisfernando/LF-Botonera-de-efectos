//! Modulo: engine/player/deck.rs
//! Proposito: un "deck" del reproductor auxiliar. Lleva el estado de la pista
//! cargada y el mando de su fuente dentro del bus. Reproduce una pista a la vez;
//! el motor usa dos decks para la pre-carga ping-pong.
//!
//! Ya NO es un `Sink` de rodio: entrega su fuente al bus `Reproductor` de la
//! consola, como el motor de efectos entrega las suyas. Lo que el `Sink` daba
//! gratis —posicion, fin de pista y pausa— lo sostiene ahora `DeckHandle`.
//!
//! El volumen tampoco esta aqui: es el fader del bus. El deck solo aplica la
//! ganancia de SU pista, que es del archivo.
use super::source::{DeckHandle, DeckSource};
use crate::engine::audio::decode::BoxSource;
use crate::engine::console::Bus;

/// Estado de un deck. `Loaded` = pre-cargado en pausa, listo para arrancar al
/// instante. `Finished` = termino de forma natural y espera relevo. `Failed` = no
/// se pudo cargar (carpeta vacia, sin clima, archivo ilegible); se trata como
/// "termino" para que el motor releve y la musica siga.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeckStatus {
    Empty,
    Loaded,
    Playing,
    Paused,
    Finished,
    Failed,
}

/// Lo que el deck recuerda de la pista cargada. Hace falta para poder
/// RECONSTRUIRLA en otra posicion: una fuente no se reposiciona, asi que un seek
/// es volver a crearla desde el punto pedido.
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

#[derive(Default)]
pub struct Deck {
    /// El mando de la fuente que esta en el bus. None = no hay nada cargado.
    handle: Option<DeckHandle>,
    status: DeckStatus,
    track: DeckTrack,
    /// Segundos que ya habian pasado al recargar por un seek: la fuente nueva
    /// cuenta desde cero, asi que sin esto el tiempo retrocederia.
    position_offset_s: f64,
}

impl Default for DeckStatus {
    fn default() -> Self {
        Self::Empty
    }
}

impl Deck {
    pub fn new() -> Self {
        Self::default()
    }

    /// Entrega la fuente al bus y la deja lista en pausa.
    pub fn load(&mut self, bus: &Bus, source: BoxSource, track: DeckTrack) {
        self.release();
        let (source, handle) = DeckSource::new(source, track.gain.max(0.0));
        bus.add(source);
        self.handle = Some(handle);
        self.track = track;
        self.position_offset_s = 0.0;
        self.status = DeckStatus::Loaded;
    }

    /// Recarga la MISMA pista desde `offset_s` (seek). Conserva si sonaba o
    /// estaba en pausa: saltar de posicion no debe arrancar ni parar nada.
    pub fn reload_at(&mut self, bus: &Bus, source: BoxSource, offset_s: f64) {
        let was_playing = self.status == DeckStatus::Playing;
        self.release();
        let (source, handle) = DeckSource::new(source, self.track.gain.max(0.0));
        bus.add(source);
        if was_playing {
            handle.play();
        }
        self.handle = Some(handle);
        self.position_offset_s = offset_s;
        self.status = if was_playing {
            DeckStatus::Playing
        } else {
            DeckStatus::Paused
        };
    }

    /// Suelta la fuente actual: se retira del bus en cuanto lo lea.
    fn release(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.stop();
        }
    }

    /// Datos de la pista cargada, para poder reconstruirla en un seek.
    pub fn track(&self) -> &DeckTrack {
        &self.track
    }

    pub fn play(&mut self) {
        if matches!(self.status, DeckStatus::Loaded | DeckStatus::Paused) {
            if let Some(handle) = &self.handle {
                handle.play();
            }
            self.status = DeckStatus::Playing;
        }
    }

    pub fn pause(&mut self) {
        if self.status == DeckStatus::Playing {
            if let Some(handle) = &self.handle {
                handle.pause();
            }
            self.status = DeckStatus::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.release();
        self.status = DeckStatus::Empty;
        self.track = DeckTrack::default();
        self.position_offset_s = 0.0;
    }

    /// Marca `Finished` una sola vez, en la transicion, si la fuente se agoto
    /// mientras reproducia. El motor lo usa para el relevo ping-pong.
    pub fn poll_finished(&mut self) -> bool {
        // `Failed` cuenta como terminado: si una pista no se pudo resolver, el
        // motor debe relevarla en vez de quedarse callado esperandola.
        let ended = self.status == DeckStatus::Playing
            && self.handle.as_ref().is_some_and(DeckHandle::is_done);
        if ended || self.status == DeckStatus::Failed {
            self.status = DeckStatus::Finished;
            return true;
        }
        false
    }

    /// La pista no se pudo cargar. Deja el deck listo para que lo releven.
    pub fn fail(&mut self) {
        self.release();
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
        self.position_offset_s
            + self.handle.as_ref().map_or(0.0, DeckHandle::position_s)
    }
    /// Solo se puede saltar de posicion en una pista con duracion conocida y de
    /// un solo archivo (una locucion son varios encadenados).
    pub fn can_seek(&self) -> bool {
        self.track.seekable && self.track.duration_s > 0.0 && !self.track.path.is_empty()
    }
}
