/// Modulo: engine/console/fader.rs
/// Proposito: FaderSource — el fader de un bus. Multiplica la señal por un
/// atomico que se puede mover mientras suena.
///
/// Parece trivial y es el punto entero de la Fase 2. Antes el volumen master era
/// un numero que CADA fuente leia y se aplicaba a si misma: no era una etapa,
/// era un acuerdo entre fuentes, y por eso el "programa" no existia en ningun
/// punto del codigo — solo era el resultado de que varias fuentes obedecieran el
/// mismo valor. Aqui es una etapa real, en un sitio real, por la que pasa toda
/// la señal del bus una sola vez.
///
/// Sin rampa, a proposito: el master ya saltaba de golpe cuando lo aplicaba cada
/// fuente, asi que suavizarlo aqui seria cambiar el sonido en la misma fase que
/// mueve la aritmetica de sitio, y ya no se sabria que causo que. Cuando el
/// fader sea visible y se arrastre de verdad (Fase 6) se decide si hace falta.
use rodio::Source;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct FaderSource<S: Source<Item = f32>> {
    inner: S,
    gain: Arc<AtomicU32>,
}

impl<S: Source<Item = f32>> FaderSource<S> {
    pub fn new(inner: S, gain: Arc<AtomicU32>) -> Self {
        Self { inner, gain }
    }
}

impl<S: Source<Item = f32>> Iterator for FaderSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let s = self.inner.next()?;
        Some(s * f32::from_bits(self.gain.load(Ordering::Relaxed)))
    }
}

impl<S: Source<Item = f32>> Source for FaderSource<S> {
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

#[cfg(test)]
mod tests {
    use super::FaderSource;
    use rodio::buffer::SamplesBuffer;
    use std::sync::atomic::AtomicU32;
    use std::sync::Arc;

    #[test]
    fn el_fader_escala_la_senal() {
        let gain = Arc::new(AtomicU32::new(0.5f32.to_bits()));
        let out: Vec<f32> = FaderSource::new(source(&[1.0, -1.0, 0.5]), gain).collect();
        assert_eq!(out, vec![0.5, -0.5, 0.25]);
    }

    /// Mover el fader afecta a lo que YA esta sonando: por eso es un atomico y
    /// se lee muestra a muestra, no una copia tomada al empezar.
    #[test]
    fn mover_el_fader_afecta_a_la_senal_en_curso() {
        let gain = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        let mut fader = FaderSource::new(source(&[1.0, 1.0, 1.0]), Arc::clone(&gain));
        assert_eq!(fader.next(), Some(1.0));
        gain.store(0.25f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
        assert_eq!(fader.next(), Some(0.25));
    }

    /// A cero el bus calla, pero la fuente sigue avanzando: bajar el fader no
    /// detiene nada ni adelanta el final de lo que suena.
    #[test]
    fn a_cero_calla_pero_no_detiene() {
        let gain = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let out: Vec<f32> = FaderSource::new(source(&[1.0, 1.0]), gain).collect();
        assert_eq!(out, vec![0.0, 0.0]);
    }

    fn source(samples: &[f32]) -> SamplesBuffer<f32> {
        SamplesBuffer::new(1, 48_000, samples.to_vec())
    }
}
