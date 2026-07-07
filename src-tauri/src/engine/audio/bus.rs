use crate::engine::audio::decode as audio_decode;
/// Módulo: master_bus.rs
/// Propósito: Master Bus de audio — todas las fuentes entran a un DynamicMixer;
/// un único LevelSource mide el PICO de la señal sumada real post-mezcla.
use crate::engine::dsp::fade::FadeRamp;
use crate::engine::audio::button::ButtonSource;
use crate::engine::audio::vu::LevelSource;
use rodio::dynamic_mixer::{self, DynamicMixerController};
use rodio::source::Zero;
use rodio::{OutputStreamHandle, Sink, Source};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::engine::audio::button::{ButtonState, ButtonStateMap};

// ─── Fuente secuencial (para PlaySequence) ────────────────────────────────────

/// Reproduce varios archivos de audio en orden como una sola fuente f32.
pub struct SequenceSource {
    queue: VecDeque<Box<dyn Source<Item = f32> + Send + 'static>>,
    current: Box<dyn Source<Item = f32> + Send + 'static>,
}

impl SequenceSource {
    pub fn from_paths(paths: &[String]) -> Option<Self> {
        let mut queue: VecDeque<Box<dyn Source<Item = f32> + Send + 'static>> = paths
            .iter()
            .filter_map(|p| audio_decode::source_from_path(p, false))
            .collect();
        let current = queue.pop_front()?;
        Some(Self { queue, current })
    }
}

impl Iterator for SequenceSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        loop {
            if let Some(s) = self.current.next() {
                return Some(s);
            }
            self.current = self.queue.pop_front()?;
        }
    }
}

impl Source for SequenceSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.current.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.current.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.current.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// ─── Master Bus ───────────────────────────────────────────────────────────────

const MIXER_CHANNELS: u16 = 2;
const MIXER_SAMPLE_RATE: u32 = 48_000;

/// Bus master: mezcla todas las fuentes en un DynamicMixer y mide el PICO real.
pub struct MasterBus {
    controller: Arc<DynamicMixerController<f32>>, // rodio::mixer() ya devuelve Arc
    master_volume: Arc<AtomicU32>,
    _sink: Sink,
}

impl MasterBus {
    /// Crea un bus conectado al OutputStreamHandle dado.
    /// Devuelve None si el Sink no puede crearse (device no disponible).
    pub fn new(
        handle: &OutputStreamHandle,
        master_level_l: Arc<AtomicU32>,
        master_level_r: Arc<AtomicU32>,
        master_volume: Arc<AtomicU32>,
    ) -> Option<Self> {
        let (ctrl, mixer) = dynamic_mixer::mixer::<f32>(MIXER_CHANNELS, MIXER_SAMPLE_RATE);
        // SIN esto el DynamicMixer devuelve None inmediatamente cuando está vacío →
        // el Sink se detiene → las fuentes añadidas después no producen audio.
        ctrl.add(Zero::<f32>::new(MIXER_CHANNELS, MIXER_SAMPLE_RATE));
        let level_src = LevelSource::new(mixer, master_level_l, master_level_r);
        let sink = Sink::try_new(handle).ok()?;
        sink.append(level_src);
        Some(Self {
            controller: ctrl,
            master_volume,
            _sink: sink,
        })
    }

    /// Añade una fuente al bus y devuelve el ButtonState para control externo.
    #[allow(clippy::too_many_arguments)]
    pub fn add_source(
        &self,
        source: Box<dyn Source<Item = f32> + Send + 'static>,
        volume: f32,
        duration: f64,
        loop_mode: bool,
        file_gain: f32,
        fade_in_s: f64,
        fade_out_stop_s: f64,
        fade_out_end_s: f64,
        position_offset_s: f64,
    ) -> ButtonState {
        let done_flag = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let vol_atomic = Arc::new(AtomicU32::new(volume.to_bits()));
        let sr = source.sample_rate();
        let ch = source.channels();
        let fade_duration = if loop_mode {
            duration
        } else {
            (duration - position_offset_s).max(0.0)
        };
        let total_samples = if fade_duration > 0.0 {
            (fade_duration * sr as f64 * ch as f64).round() as usize
        } else {
            0
        };
        let (fade, fade_out_flag) = FadeRamp::new(
            fade_in_s,
            fade_out_stop_s,
            fade_out_end_s,
            sr,
            ch,
            total_samples,
            loop_mode,
        );
        self.controller.add(ButtonSource {
            inner: source,
            stop_flag: Arc::clone(&stop_flag),
            done_flag: Arc::clone(&done_flag),
            file_gain,
            volume: Arc::clone(&vol_atomic),
            master_volume: Arc::clone(&self.master_volume),
            fade,
        });
        ButtonState {
            done_flag,
            stop_flag,
            fade_out_flag,
            volume: vol_atomic,
            start_time: Instant::now(),
            position_offset_s,
            duration,
            loop_mode,
        }
    }
}
