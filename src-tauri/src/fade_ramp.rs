/// Módulo: fade_ramp.rs
/// Propósito: rampa lineal de ganancia para fade-in y fade-out en ButtonSource.
/// Desacoplado de master_button.rs para mantener ambos bajo el límite de 200 líneas.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct FadeRamp {
    /// Muestras totales del fade-in (0 = desactivado).
    fade_in_total: usize,
    /// Muestras totales del fade-out-al-parar (0 = desactivado).
    fade_out_stop_total: usize,
    /// Muestras totales del fade-out al final natural (0 = desactivado).
    fade_out_end_total: usize,
    /// Muestras totales de la pista (para detectar el inicio del fade final).
    track_total: usize,
    /// Posición actual (número de muestra procesada).
    pos: usize,
    /// Señal de que se pidió detener con fundido (set por ButtonState).
    pub fade_out_flag: Option<Arc<AtomicBool>>,
    /// Posición de la muestra en la que comenzó el fade-out-al-parar.
    fade_out_stop_start: Option<usize>,
}

impl FadeRamp {
    /// Crea la rampa a partir de tiempos en segundos y parámetros del stream.
    pub fn new(
        fade_in_s: f64,
        fade_out_stop_s: f64,
        fade_out_end_s: f64,
        sample_rate: u32,
        channels: u16,
        total_samples: usize,
        loop_mode: bool,
    ) -> (Self, Option<Arc<AtomicBool>>) {
        let sr = sample_rate as f64 * channels as f64;
        let to_samples = |s: f64| (s * sr).round() as usize;
        let fade_out_flag = if fade_out_stop_s > 0.0 && !loop_mode {
            Some(Arc::new(AtomicBool::new(false)))
        } else {
            None
        };
        let ramp = Self {
            fade_in_total: to_samples(fade_in_s),
            fade_out_stop_total: to_samples(fade_out_stop_s),
            fade_out_end_total: if loop_mode { 0 } else { to_samples(fade_out_end_s) },
            track_total: total_samples,
            pos: 0,
            fade_out_flag: fade_out_flag.clone(),
            fade_out_stop_start: None,
        };
        (ramp, fade_out_flag)
    }

    /// Avanza la posición y devuelve el factor de ganancia [0.0, 1.0] para la muestra actual.
    /// Devuelve `None` cuando el fade-out ha concluido (la fuente debe terminar).
    pub fn next_gain(&mut self) -> Option<f32> {
        // ── Fade-out-al-parar ──────────────────────────────────────────────────
        if let Some(flag) = &self.fade_out_flag {
            if flag.load(Ordering::Relaxed) && self.fade_out_stop_start.is_none() {
                self.fade_out_stop_start = Some(self.pos);
            }
        }
        if let Some(start) = self.fade_out_stop_start {
            let elapsed = self.pos.saturating_sub(start);
            if self.fade_out_stop_total == 0 || elapsed >= self.fade_out_stop_total {
                return None; // Fundido completado: señal de fin
            }
            let gain = 1.0 - elapsed as f32 / self.fade_out_stop_total as f32;
            self.pos += 1;
            return Some(gain.clamp(0.0, 1.0));
        }

        // ── Fade-out al final natural ──────────────────────────────────────────
        if self.fade_out_end_total > 0 && self.track_total > 0 {
            let end_start = self.track_total.saturating_sub(self.fade_out_end_total);
            if self.pos >= end_start {
                let elapsed = self.pos - end_start;
                if elapsed >= self.fade_out_end_total {
                    self.pos += 1;
                    return Some(0.0);
                }
                let gain = 1.0 - elapsed as f32 / self.fade_out_end_total as f32;
                self.pos += 1;
                return Some(gain.clamp(0.0, 1.0));
            }
        }

        // ── Fade-in ────────────────────────────────────────────────────────────
        let gain = if self.fade_in_total > 0 && self.pos < self.fade_in_total {
            self.pos as f32 / self.fade_in_total as f32
        } else {
            1.0
        };
        self.pos += 1;
        Some(gain.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(fade_in_s: f64, fade_out_stop_s: f64, fade_out_end_s: f64, sr: u32, ch: u16, total: usize) -> FadeRamp {
        let (ramp, _) = FadeRamp::new(fade_in_s, fade_out_stop_s, fade_out_end_s, sr, ch, total, false);
        ramp
    }

    #[test]
    fn no_fade_always_one() {
        let mut r = build(0.0, 0.0, 0.0, 48000, 1, 48000);
        for _ in 0..1000 {
            assert_eq!(r.next_gain(), Some(1.0));
        }
    }

    #[test]
    fn fade_in_ramps_up() {
        let mut r = build(1.0, 0.0, 0.0, 10, 1, 100);
        let g0 = r.next_gain().unwrap(); // pos=0 → 0/10
        assert_eq!(g0, 0.0);
        let g5 = (1..5).map(|_| r.next_gain().unwrap()).last().unwrap();
        assert!(g5 > 0.0 && g5 < 1.0);
        // Después de 10 muestras debe ser 1.0
        for _ in 5..10 { r.next_gain(); }
        assert_eq!(r.next_gain(), Some(1.0));
    }

    #[test]
    fn fade_out_stop_returns_none_when_done() {
        // 1.0s × 10 sr × 1 ch = 10 muestras de rampa
        let (mut r, flag) = FadeRamp::new(0.0, 1.0, 0.0, 10, 1, 1000, false);
        flag.unwrap().store(true, Ordering::Relaxed);
        // Las 10 muestras de la rampa deben ser Some
        let last = (0..10).map(|_| r.next_gain()).last().unwrap();
        assert!(last.is_some());
        // La muestra 11 (elapsed=10 >= total=10) → None
        assert!(r.next_gain().is_none());
    }
}
