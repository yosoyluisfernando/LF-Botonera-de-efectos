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
use super::deck_track::{DeckStatus, DeckTrack};
use super::prefetch::deck_source;
use super::source::{DeckHandle, DeckSource};
use crate::engine::audio::decode::BoxSource;
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::Bus;
use std::sync::{Arc, Mutex};

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

    /// El bus murio (cambio de tarjeta) y con el la fuente: se rehace la MISMA
    /// pista en el bus nuevo, por el segundo en el que iba. El deck conserva su
    /// estado, asi que lo que sonaba sigue sonando y lo que estaba en pausa sigue
    /// en pausa.
    ///
    /// Una locucion no se puede reposicionar (son varios archivos encadenados),
    /// asi que se deja caer: se marca terminada para que el motor releve y la
    /// lista siga, en vez de callarse esperandola.
    pub fn reattach(&mut self, bus: &Bus, cache: &Arc<Mutex<PreloadCache>>) {
        if self.status == DeckStatus::Empty || self.track.path.is_empty() {
            return;
        }
        let at = self.position_s();
        if !self.track.seekable {
            self.fail();
            return;
        }
        let from = (self.track.cue_start_s + at).max(0.0);
        let Some(source) = deck_source(cache, &self.track.path, from, self.track.cue_end_s) else {
            self.fail();
            return;
        };
        self.reload_at(bus, source, at);
    }

    /// Suelta una pista pre-cargada que no llego a sonar. El ping-pong la volvera
    /// a preparar cuando toque: no hay que rehacerla ahora.
    pub fn release_preloaded(&mut self) {
        if self.status == DeckStatus::Loaded {
            self.stop();
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
