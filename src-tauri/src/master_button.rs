/// Modulo: master_button.rs
/// Proposito: estado y fuente controlable de un boton dentro del bus master.
use rodio::Source;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type ButtonStateMap = HashMap<String, Vec<ButtonState>>;

/// Estado de un boton en reproduccion: flags atomicos de control + tiempo real.
pub struct ButtonState {
    pub done_flag: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
    pub volume: Arc<AtomicU32>,
    pub start_time: Instant,
    pub duration: f64,
    pub loop_mode: bool,
}

impl ButtonState {
    pub fn is_done(&self) -> bool {
        self.done_flag.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
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

    /// Posicion de reproduccion en segundos desde el inicio.
    pub fn position(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

/// Envuelve Source<Item=f32> con control atomico de volumen y stop.
/// Ganancia en 3 capas: `file_gain` (normalizacion/dB del editor, fija) ×
/// `volume` (trim del boton, en vivo) × `master_volume` (global).
pub struct ButtonSource {
    pub inner: Box<dyn Source<Item = f32> + Send + 'static>,
    pub stop_flag: Arc<AtomicBool>,
    pub done_flag: Arc<AtomicBool>,
    pub file_gain: f32,
    pub volume: Arc<AtomicU32>,
    pub master_volume: Arc<AtomicU32>,
}

impl Iterator for ButtonSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.stop_flag.load(Ordering::Relaxed) {
            self.done_flag.store(true, Ordering::Relaxed);
            return None;
        }
        match self.inner.next() {
            Some(s) => {
                let local = f32::from_bits(self.volume.load(Ordering::Relaxed));
                let master = f32::from_bits(self.master_volume.load(Ordering::Relaxed));
                Some(s * self.file_gain * local * master)
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
    use super::ButtonState;
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
            done_flag: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            start_time: Instant::now() - elapsed,
            duration,
            loop_mode,
        }
    }
}
