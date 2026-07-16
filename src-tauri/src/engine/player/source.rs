//! Modulo: engine/player/source.rs
//! Proposito: la fuente de un deck dentro del bus, y el mando a distancia para
//! controlarla desde fuera.
//!
//! Sustituye al `Sink` de rodio, del que el reproductor dependia para tres cosas
//! que ahora hay que resolver a mano, porque una fuente metida en un mixer ya no
//! se puede "preguntar":
//!
//! - **Posicion** (`sink.get_pos`): se cuentan las muestras consumidas.
//! - **Termino** (`sink.empty`): un flag que la fuente marca al agotarse.
//! - **Pausa** (`sink.pause`): un flag; en pausa devuelve silencio SIN avanzar la
//!   fuente, asi que al reanudar sigue por donde iba.
//!
//! El volumen no esta aqui: es el fader del bus `Reproductor`. Lo unico que la
//! fuente se aplica a si misma es la ganancia de SU pista, que es del archivo y
//! no del reproductor.
use crate::engine::audio::decode::BoxSource;
use rodio::Source;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// El mando de una fuente que ya esta sonando en el bus. Todo atomicos: se
/// consulta y se mueve desde el hilo del reproductor sin tocar el audio.
#[derive(Clone)]
pub struct DeckHandle {
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
    samples: Arc<AtomicU64>,
    sample_rate: u32,
    channels: u16,
}

impl DeckHandle {
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }
    pub fn play(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }
    /// La fuente se retira del bus en cuanto lo lea. No hay vuelta atras.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
    /// Se agoto sola o la pararon.
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Relaxed)
    }
    /// Segundos reproducidos, contando muestras. En pausa no avanza, que es justo
    /// lo que se espera de un contador.
    pub fn position_s(&self) -> f64 {
        let per_second = self.sample_rate as f64 * self.channels.max(1) as f64;
        if per_second <= 0.0 {
            return 0.0;
        }
        self.samples.load(Ordering::Relaxed) as f64 / per_second
    }
}

pub struct DeckSource {
    inner: BoxSource,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
    samples: Arc<AtomicU64>,
    gain: f32,
    channels: u16,
    sample_rate: u32,
}

impl DeckSource {
    /// Envuelve la fuente y devuelve el mando para controlarla. `gain` es la
    /// ganancia de la pista (cue/normalizacion), no el volumen del reproductor.
    pub fn new(inner: BoxSource, gain: f32) -> (Self, DeckHandle) {
        let channels = inner.channels().max(1);
        let sample_rate = inner.sample_rate();
        let handle = DeckHandle {
            stop: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(true)),
            done: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(AtomicU64::new(0)),
            sample_rate,
            channels,
        };
        let source = Self {
            inner,
            stop: Arc::clone(&handle.stop),
            paused: Arc::clone(&handle.paused),
            done: Arc::clone(&handle.done),
            samples: Arc::clone(&handle.samples),
            gain,
            channels,
            sample_rate,
        };
        (source, handle)
    }
}

impl Iterator for DeckSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.stop.load(Ordering::Relaxed) {
            self.done.store(true, Ordering::Relaxed);
            return None;
        }
        // En pausa se devuelve silencio pero NO se pide muestra a la fuente: asi
        // al reanudar sigue por donde iba. Devolver None la retiraria del bus y
        // pausar seria en realidad parar.
        if self.paused.load(Ordering::Relaxed) {
            return Some(0.0);
        }
        match self.inner.next() {
            Some(s) => {
                self.samples.fetch_add(1, Ordering::Relaxed);
                Some(s * self.gain)
            }
            None => {
                self.done.store(true, Ordering::Relaxed);
                None
            }
        }
    }
}

impl Source for DeckSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod source_tests;
