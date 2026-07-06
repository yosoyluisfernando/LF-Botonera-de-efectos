/// Modulo: playback_source.rs
/// Proposito: construir fuentes de reproduccion normales y con seek.
use crate::engine::audio::decode::BoxSource;
use crate::engine::cache::preload::{self as preload_cache, PreloadCache};
use rodio::Source;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn build(
    cache: &Arc<Mutex<PreloadCache>>,
    path: &str,
    loop_mode: bool,
    cue_start_s: f64,
    cue_end_s: Option<f64>,
) -> Option<BoxSource> {
    preload_cache::build_play_source(cache, path, loop_mode, cue_start_s, cue_end_s)
}

pub fn build_seek(
    cache: &Arc<Mutex<PreloadCache>>,
    path: &str,
    loop_mode: bool,
    base_cue_start_s: f64,
    position_s: f64,
    cue_end_s: Option<f64>,
) -> Option<BoxSource> {
    let start = base_cue_start_s + position_s.max(0.0);
    if !loop_mode || position_s <= 0.005 {
        return build(cache, path, loop_mode, start, cue_end_s);
    }
    let first = build(cache, path, false, start, cue_end_s)?;
    let looped = build(cache, path, true, base_cue_start_s, cue_end_s)?;
    Some(Box::new(SeekLoopSource::new(first, looped)))
}

struct SeekLoopSource {
    first: Option<BoxSource>,
    looped: BoxSource,
    channels: u16,
    sample_rate: u32,
}

impl SeekLoopSource {
    fn new(first: BoxSource, looped: BoxSource) -> Self {
        let channels = first.channels();
        let sample_rate = first.sample_rate();
        Self {
            first: Some(first),
            looped,
            channels,
            sample_rate,
        }
    }
}

impl Iterator for SeekLoopSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if let Some(first) = self.first.as_mut() {
            if let Some(sample) = first.next() {
                return Some(sample);
            }
            self.first = None;
        }
        self.looped.next()
    }
}

impl Source for SeekLoopSource {
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
