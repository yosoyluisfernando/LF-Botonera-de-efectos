/// Módulo: master_bus.rs
/// Propósito: Master Bus de audio — todas las fuentes entran a un DynamicMixer;
/// un único LevelSource mide el PICO de la señal sumada real post-mezcla.
/// Equivale al canal master de una consola de audio real. No usa fórmulas
/// de aproximación: mide directamente el PCM que sale al dispositivo.

use crate::vu_meter::LevelSource;
use rodio::dynamic_mixer::{self, DynamicMixerController};
use rodio::source::Zero;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ─── Tipos públicos ───────────────────────────────────────────────────────────

pub type ButtonStateMap = HashMap<String, Vec<ButtonState>>;

/// Estado de un botón en reproducción: flags atómicos de control + info de tiempo.
pub struct ButtonState {
    pub done_flag:  Arc<AtomicBool>,
    pub stop_flag:  Arc<AtomicBool>,
    pub volume:     Arc<AtomicU32>,
    pub start_time: Instant,
    pub duration:   f64,
}

impl ButtonState {
    pub fn is_done(&self) -> bool { self.done_flag.load(Ordering::Relaxed) }

    pub fn stop(&self) { self.stop_flag.store(true, Ordering::Relaxed); }

    pub fn set_volume(&self, v: f32) {
        self.volume.store(v.to_bits(), Ordering::Relaxed);
    }

    /// Segundos restantes (0.0 para bucles o cuando ya terminó).
    pub fn remaining(&self) -> f64 {
        if self.duration <= 0.0 { return 0.0; }
        (self.duration - self.start_time.elapsed().as_secs_f64()).max(0.0)
    }

    /// Posición de reproducción en segundos desde el inicio.
    pub fn position(&self) -> f64 { self.start_time.elapsed().as_secs_f64() }
}

// ─── Fuente por botón ─────────────────────────────────────────────────────────

/// Envuelve Source<Item=f32> (boxeado) con control atómico de volumen y stop.
/// Cuando la fuente se agota o se activa stop_flag, activa done_flag y devuelve None.
pub struct ButtonSource {
    inner:     Box<dyn Source<Item = f32> + Send + 'static>,
    stop_flag: Arc<AtomicBool>,
    done_flag: Arc<AtomicBool>,
    volume:    Arc<AtomicU32>,
}

impl Iterator for ButtonSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.stop_flag.load(Ordering::Relaxed) {
            self.done_flag.store(true, Ordering::Relaxed);
            return None;
        }
        match self.inner.next() {
            Some(s) => Some(s * f32::from_bits(self.volume.load(Ordering::Relaxed))),
            None    => { self.done_flag.store(true, Ordering::Relaxed); None }
        }
    }
}

impl Source for ButtonSource {
    fn current_frame_len(&self) -> Option<usize>   { self.inner.current_frame_len() }
    fn channels(&self)          -> u16              { self.inner.channels() }
    fn sample_rate(&self)       -> u32              { self.inner.sample_rate() }
    fn total_duration(&self)    -> Option<Duration> { None }
}

// ─── Fuente secuencial (para PlaySequence) ────────────────────────────────────

/// Reproduce varios archivos de audio en orden como una sola fuente f32.
pub struct SequenceSource {
    queue:       VecDeque<Box<dyn Source<Item = f32> + Send + 'static>>,
    current:     Box<dyn Source<Item = f32> + Send + 'static>,
}

impl SequenceSource {
    pub fn from_paths(paths: &[String]) -> Option<Self> {
        let mut queue: VecDeque<Box<dyn Source<Item = f32> + Send + 'static>> = paths.iter()
            .filter_map(|p| File::open(p).ok())
            .filter_map(|f| Decoder::new(BufReader::new(f)).ok())
            .map(|d| -> Box<dyn Source<Item = f32> + Send + 'static> {
                Box::new(d.convert_samples::<f32>())
            })
            .collect();
        let current = queue.pop_front()?;
        Some(Self { queue, current })
    }
}

impl Iterator for SequenceSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        loop {
            if let Some(s) = self.current.next() { return Some(s); }
            self.current = self.queue.pop_front()?;
        }
    }
}

impl Source for SequenceSource {
    fn current_frame_len(&self) -> Option<usize>   { self.current.current_frame_len() }
    fn channels(&self)          -> u16              { self.current.channels() }
    fn sample_rate(&self)       -> u32              { self.current.sample_rate() }
    fn total_duration(&self)    -> Option<Duration> { None }
}

// ─── Master Bus ───────────────────────────────────────────────────────────────

const MIXER_CHANNELS:     u16 = 2;
const MIXER_SAMPLE_RATE:  u32 = 48_000;

/// Bus master: mezcla todas las fuentes en un DynamicMixer y mide el PICO real.
pub struct MasterBus {
    controller: Arc<DynamicMixerController<f32>>,  // rodio::mixer() ya devuelve Arc
    _sink:      Sink,
}

impl MasterBus {
    /// Crea un bus conectado al OutputStreamHandle dado.
    /// Devuelve None si el Sink no puede crearse (device no disponible).
    pub fn new(
        handle:         &OutputStreamHandle,
        master_level_l: Arc<AtomicU32>,
        master_level_r: Arc<AtomicU32>,
    ) -> Option<Self> {
        let (ctrl, mixer) = dynamic_mixer::mixer::<f32>(MIXER_CHANNELS, MIXER_SAMPLE_RATE);
        // SIN esto el DynamicMixer devuelve None inmediatamente cuando está vacío →
        // el Sink se detiene → las fuentes añadidas después no producen audio.
        ctrl.add(Zero::<f32>::new(MIXER_CHANNELS, MIXER_SAMPLE_RATE));
        let level_src = LevelSource::new(mixer, master_level_l, master_level_r);
        let sink      = Sink::try_new(handle).ok()?;
        sink.append(level_src);
        Some(Self { controller: ctrl, _sink: sink })
    }

    /// Añade una fuente al bus y devuelve el ButtonState para control externo.
    pub fn add_source(
        &self,
        source:   Box<dyn Source<Item = f32> + Send + 'static>,
        volume:   f32,
        duration: f64,
    ) -> ButtonState {
        let done_flag  = Arc::new(AtomicBool::new(false));
        let stop_flag  = Arc::new(AtomicBool::new(false));
        let vol_atomic = Arc::new(AtomicU32::new(volume.to_bits()));
        self.controller.add(ButtonSource {
            inner: source,
            stop_flag: Arc::clone(&stop_flag),
            done_flag: Arc::clone(&done_flag),
            volume:    Arc::clone(&vol_atomic),
        });
        ButtonState { done_flag, stop_flag, volume: vol_atomic, start_time: Instant::now(), duration }
    }
}
