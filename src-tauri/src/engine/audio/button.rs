/// Modulo: master_button.rs
/// Proposito: estado y fuente controlable de un boton dentro del bus master.
use crate::engine::dsp::fade::FadeRamp;
use rodio::Source;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type ButtonStateMap = HashMap<String, Vec<ButtonState>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackGroup {
    Main,
    Fixed,
}

/// Estado de un boton en reproduccion: flags atomicos de control + tiempo real.
pub struct ButtonState {
    pub group: PlaybackGroup,
    pub done_flag: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
    /// Presente solo cuando el ButtonSource fue creado con fade_out_stop > 0.
    /// Activarlo inicia el fundido antes del corte definitivo.
    pub fade_out_flag: Option<Arc<AtomicBool>>,
    pub volume: Arc<AtomicU32>,
    pub start_time: Instant,
    pub position_offset_s: f64,
    pub duration: f64,
    pub loop_mode: bool,
}

impl ButtonState {
    pub fn is_done(&self) -> bool {
        self.done_flag.load(Ordering::Relaxed)
    }

    /// Detiene con fundido si está configurado; si no, corte inmediato.
    pub fn stop(&self) {
        if let Some(flag) = &self.fade_out_flag {
            flag.store(true, Ordering::Relaxed);
        } else {
            self.stop_flag.store(true, Ordering::Relaxed);
        }
    }

    /// Siempre corte inmediato (limpieza interna y secuencias).
    pub fn stop_immediate(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn set_volume(&self, v: f32) {
        self.volume.store(v.to_bits(), Ordering::Relaxed);
    }

    /// Segundos restantes honestos. En loop, vuelve a contar cada vuelta.
    pub fn remaining(&self) -> f64 {
        if self.duration <= 0.0 {
            return 0.0;
        }
        if self.loop_mode {
            let cycle_pos = self.position() % self.duration;
            let remaining = self.duration - cycle_pos;
            if remaining <= 0.005 {
                self.duration
            } else {
                remaining
            }
        } else {
            (self.duration - self.position()).max(0.0)
        }
    }

    pub fn position(&self) -> f64 {
        self.position_offset_s + self.start_time.elapsed().as_secs_f64()
    }
}

/// Envuelve Source<Item=f32> con control atomico de volumen, stop y fade.
/// Ganancia en 3 capas: `file_gain` × `volume` (trim) × `master_volume`.
pub struct ButtonSource {
    pub inner: Box<dyn Source<Item = f32> + Send + 'static>,
    pub stop_flag: Arc<AtomicBool>,
    pub done_flag: Arc<AtomicBool>,
    pub file_gain: f32,
    pub volume: Arc<AtomicU32>,
    pub master_volume: Arc<AtomicU32>,
    pub fade: FadeRamp,
}

impl Iterator for ButtonSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.stop_flag.load(Ordering::Relaxed) {
            self.done_flag.store(true, Ordering::Relaxed);
            return None;
        }
        let fade_gain = match self.fade.next_gain() {
            Some(g) => g,
            None => {
                self.done_flag.store(true, Ordering::Relaxed);
                return None;
            }
        };
        match self.inner.next() {
            Some(s) => {
                let local = f32::from_bits(self.volume.load(Ordering::Relaxed));
                let master = f32::from_bits(self.master_volume.load(Ordering::Relaxed));
                Some(s * self.file_gain * local * master * fade_gain)
            }
            None => {
                self.done_flag.store(true, Ordering::Relaxed);
                None
            }
        }
    }
}

impl Source for ButtonSource {
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
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{ButtonState, PlaybackGroup};
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[test]
    fn non_loop_remaining_stops_at_zero() {
        let state = state_started_ago(12.0, false, Duration::from_secs(15));
        assert_eq!(state.remaining(), 0.0);
    }

    #[test]
    fn loop_remaining_wraps_each_cycle() {
        let state = state_started_ago(10.0, true, Duration::from_secs(12));
        let remaining = state.remaining();
        assert!(remaining > 7.5 && remaining <= 8.0);
    }

    fn state_started_ago(duration: f64, loop_mode: bool, elapsed: Duration) -> ButtonState {
        ButtonState {
            group: PlaybackGroup::Main,
            done_flag: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            fade_out_flag: None,
            volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            start_time: Instant::now() - elapsed,
            position_offset_s: 0.0,
            duration,
            loop_mode,
        }
    }
}
