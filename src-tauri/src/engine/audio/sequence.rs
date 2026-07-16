/// Modulo: engine/audio/sequence.rs
/// Proposito: SequenceSource — reproduce varios archivos en orden como una sola
/// fuente. Es lo que hace que una locucion (hora, temperatura) sea UNA cosa para
/// el bus. Como es un solo flujo continuo, no se puede reposicionar dentro: de
/// ahi que las locuciones no admitan salto de posicion.
use crate::engine::audio::decode as audio_decode;
use rodio::Source;
use std::collections::VecDeque;
use std::time::Duration;

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
