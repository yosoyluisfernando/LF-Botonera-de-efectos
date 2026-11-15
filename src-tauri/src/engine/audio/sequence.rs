/// Modulo: engine/audio/sequence.rs
/// Proposito: SequenceSource — reproduce varios archivos en orden como una sola
/// fuente. Es lo que hace que una locucion (hora, temperatura) sea UNA cosa para
/// el bus. Como es un solo flujo continuo, no se puede reposicionar dentro: de
/// ahi que las locuciones no admitan salto de posicion.
use crate::engine::audio::decode as audio_decode;
use rodio::source::UniformSourceIterator;
use rodio::Source;
use std::collections::VecDeque;
use std::time::Duration;

type SequencePart = UniformSourceIterator<audio_decode::BoxSource, f32>;
const CHANNELS: u16 = 2;
const SAMPLE_RATE: u32 = 48_000;

pub struct SequenceSource {
    queue: VecDeque<audio_decode::BoxSource>,
    current: SequencePart,
}

impl SequenceSource {
    pub fn from_paths(paths: &[String]) -> Option<Self> {
        let mut queue: VecDeque<_> = paths
            .iter()
            .filter_map(|p| audio_decode::source_from_path(p, false))
            .collect();
        let current = UniformSourceIterator::new(queue.pop_front()?, CHANNELS, SAMPLE_RATE);
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
            self.current = UniformSourceIterator::new(self.queue.pop_front()?, CHANNELS, SAMPLE_RATE);
        }
    }
}

impl Source for SequenceSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        CHANNELS
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
