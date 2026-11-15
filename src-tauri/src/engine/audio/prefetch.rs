//! PCM compartido por los motores: el dispositivo consume sin esperar al productor.
use super::decode::BoxSource;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::vec::IntoIter;

const AHEAD_S: usize = 10;
const CHUNK: usize = 4096;

pub enum PcmRead {
    Sample(f32),
    Pending,
    End,
}

pub enum PlaybackSource {
    InMemory(BoxSource),
    Buffered(PrefetchSource),
}

impl PlaybackSource {
    pub fn buffered(inner: BoxSource) -> Option<Self> {
        PrefetchSource::new(inner).map(Self::Buffered)
    }

    pub fn read(&mut self) -> PcmRead {
        match self {
            Self::InMemory(inner) => inner.next().map_or(PcmRead::End, PcmRead::Sample),
            Self::Buffered(inner) => inner.read(),
        }
    }

    pub fn channels(&self) -> u16 {
        match self {
            Self::InMemory(inner) => inner.channels(),
            Self::Buffered(inner) => inner.channels,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        match self {
            Self::InMemory(inner) => inner.sample_rate(),
            Self::Buffered(inner) => inner.sample_rate,
        }
    }

    pub fn current_frame_len(&self) -> Option<usize> {
        match self {
            Self::InMemory(inner) => inner.current_frame_len(),
            Self::Buffered(_) => None,
        }
    }

    #[cfg(test)]
    pub fn queued_stereo(batch: Vec<f32>) -> (Self, SyncSender<Vec<f32>>) {
        let (tx, rx) = sync_channel(1);
        let source = PrefetchSource {
            rx,
            batch: batch.into_iter(),
            channels: 2,
            sample_rate: 48_000,
            silence_remaining: 0,
        };
        (Self::Buffered(source), tx)
    }
}

pub struct PrefetchSource {
    rx: Receiver<Vec<f32>>,
    batch: IntoIter<f32>,
    channels: u16,
    sample_rate: u32,
    silence_remaining: u16,
}

impl PrefetchSource {
    fn new(inner: BoxSource) -> Option<Self> {
        let channels = inner.channels().max(1);
        let sample_rate = inner.sample_rate().max(1);
        let frame = usize::from(channels);
        let chunk = (CHUNK / frame).max(1) * frame;
        let capacity = (sample_rate as usize * frame * AHEAD_S / chunk).max(1);
        let (tx, rx) = sync_channel(capacity);
        std::thread::Builder::new()
            .name("pcm-decode".into())
            .spawn(move || fill(inner, tx, chunk, frame))
            .ok()?;
        // Solo el hilo de control espera la primera entrega, antes de montar en el bus.
        let batch = rx.recv().ok()?.into_iter();
        Some(Self {
            rx,
            batch,
            channels,
            sample_rate,
            silence_remaining: 0,
        })
    }

    fn read(&mut self) -> PcmRead {
        if self.silence_remaining != 0 {
            self.silence_remaining -= 1;
            return PcmRead::Pending;
        }
        if let Some(sample) = self.batch.next() {
            return PcmRead::Sample(sample);
        }
        match self.rx.try_recv() {
            Ok(batch) => {
                self.batch = batch.into_iter();
                self.batch.next().map_or(PcmRead::End, PcmRead::Sample)
            }
            Err(TryRecvError::Empty) => {
                // Volver a consultar al empezar el siguiente frame, nunca entre canales.
                self.silence_remaining = self.channels - 1;
                PcmRead::Pending
            }
            Err(TryRecvError::Disconnected) => PcmRead::End,
        }
    }
}

fn fill(mut inner: BoxSource, tx: SyncSender<Vec<f32>>, chunk: usize, frame: usize) {
    loop {
        let mut batch = Vec::with_capacity(chunk);
        batch.extend(inner.by_ref().take(chunk));
        let finished = batch.len() < chunk;
        batch.truncate(batch.len() / frame * frame);
        if !batch.is_empty() && tx.send(batch).is_err() {
            return;
        }
        if finished {
            return;
        }
    }
}

#[cfg(test)]
#[path = "prefetch_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "prefetch_real_tests.rs"]
mod real_tests;
