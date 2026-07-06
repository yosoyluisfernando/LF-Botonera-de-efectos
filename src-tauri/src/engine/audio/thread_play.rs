/// Modulo: audio_thread_play.rs
/// Proposito: operaciones auxiliares usadas por el hilo de audio.
use crate::engine::audio::ops::{self as audio_ops, stop_removed};
use crate::engine::audio::bus::{ButtonStateMap, MasterBus, SequenceSource};
use crate::domain::playback::source as playback_source;
use crate::engine::cache::preload::PreloadCache;
use std::sync::{Arc, Mutex};

pub struct PlayArgs {
    pub id: String,
    pub path: String,
    pub volume: f32,
    pub duration: f64,
    pub loop_mode: bool,
    pub stop_other: bool,
    pub overlap: bool,
    pub restart: bool,
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    pub file_gain: f32,
    pub fade_in_s: f64,
    pub fade_out_stop_s: f64,
    pub fade_out_end_s: f64,
    pub position_offset_s: f64,
}

pub fn play_file(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&MasterBus>,
    cache: &Arc<Mutex<PreloadCache>>,
    args: PlayArgs,
) -> bool {
    let Some(bus) = bus else { return false };
    let mut states = states.lock().unwrap();
    if args.stop_other {
        if args.fade_out_stop_s > 0.0 {
            audio_ops::fade_stop_other_ids(&mut states, &args.id);
        } else {
            audio_ops::stop_other_ids(&mut states, &args.id);
        }
    }
    if audio_ops::should_skip_existing(&mut states, &args.id, args.overlap, args.restart) {
        return false;
    }
    let Some(source) = playback_source::build(
        cache,
        &args.path,
        args.loop_mode,
        args.cue_start_s,
        args.cue_end_s,
    ) else {
        return false;
    };
    let btn_state = bus.add_source(
        source,
        args.volume,
        args.duration,
        args.loop_mode,
        args.file_gain,
        args.fade_in_s,
        args.fade_out_stop_s,
        args.fade_out_end_s,
        args.position_offset_s,
    );
    states.entry(args.id).or_default().push(btn_state);
    true
}

pub fn play_sequence(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&MasterBus>,
    id: String,
    paths: Vec<String>,
    volume: f32,
    duration: f64,
) {
    let Some(bus) = bus else { return };
    let mut states = states.lock().unwrap();
    stop_removed(states.remove(&id));
    if let Some(seq) = SequenceSource::from_paths(&paths) {
        let btn_state = bus.add_source(
            Box::new(seq),
            volume,
            duration,
            false,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        states.entry(id).or_default().push(btn_state);
    }
}
