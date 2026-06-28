/// Modulo: audio_thread.rs
/// Proposito: ejecutar el motor de audio en un hilo dedicado.
use crate::audio_command::AudioCommand;
use crate::audio_device::AudioDeviceRuntime;
use crate::audio_ops::{self, stop_removed};
use crate::master_bus::{ButtonStateMap, MasterBus, SequenceSource};
use crate::preload_cache::{self, PreloadCache};
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    master_l: Arc<AtomicU32>,
    master_r: Arc<AtomicU32>,
    master_volume: Arc<AtomicU32>,
    cache: Arc<Mutex<PreloadCache>>,
) {
    thread::spawn(move || run(rx, states, master_l, master_r, master_volume, cache));
}

fn run(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    master_l: Arc<AtomicU32>,
    master_r: Arc<AtomicU32>,
    master_volume: Arc<AtomicU32>,
    cache: Arc<Mutex<PreloadCache>>,
) {
    let mut device = AudioDeviceRuntime::new();
    let mut device_pre = AudioDeviceRuntime::new();
    let pre_volume = Arc::new(AtomicU32::new(1.0f32.to_bits()));
    let pre_l = Arc::new(AtomicU32::new(0));
    let pre_r = Arc::new(AtomicU32::new(0));

    for cmd in rx {
        audio_ops::purge_done(&states);
        match cmd {
            AudioCommand::SetDevice { device_name } => {
                device.set_device(&states, &master_l, &master_r, &master_volume, device_name, true);
            }
            AudioCommand::SetPreDevice { device_name } => {
                if device_name.is_empty() {
                    device_pre.clear();
                } else {
                    device_pre.set_device(&states, &pre_l, &pre_r, &pre_volume, device_name, false);
                }
            }
            AudioCommand::Play {
                id, path, volume, duration, loop_mode,
                stop_other, overlap, restart,
                cue_start_s, cue_end_s, file_gain, to_pre,
                fade_in_s, fade_out_stop_s, fade_out_end_s,
            } => {
                let bus = if to_pre {
                    device_pre.bus().or_else(|| device.bus())
                } else {
                    device.bus()
                };
                play_file(
                    &states, bus, &cache,
                    PlayArgs {
                        id, path, volume, duration, loop_mode,
                        stop_other, overlap, restart,
                        cue_start_s, cue_end_s, file_gain,
                        fade_in_s, fade_out_stop_s, fade_out_end_s,
                    },
                );
            }
            AudioCommand::Stop { id } => audio_ops::stop_id(&states, &id),
            AudioCommand::StopFade { id } => audio_ops::fade_stop_id(&states, &id),
            AudioCommand::StopAll => audio_ops::stop_all(&states),
            AudioCommand::StopAllFade => audio_ops::fade_stop_all(&states),
            AudioCommand::SetVolume { id, volume } => audio_ops::set_volume(&states, &id, volume),
            AudioCommand::PlaySequence { id, paths, volume, duration } => {
                play_sequence(&states, device.bus(), id, paths, volume, duration);
            }
        }
    }
}

struct PlayArgs {
    id: String,
    path: String,
    volume: f32,
    duration: f64,
    loop_mode: bool,
    stop_other: bool,
    overlap: bool,
    restart: bool,
    cue_start_s: f64,
    cue_end_s: Option<f64>,
    file_gain: f32,
    fade_in_s: f64,
    fade_out_stop_s: f64,
    fade_out_end_s: f64,
}

fn play_file(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&MasterBus>,
    cache: &Arc<Mutex<PreloadCache>>,
    args: PlayArgs,
) {
    let Some(bus) = bus else { return; };
    let mut states = states.lock().unwrap();
    if args.stop_other {
        if args.fade_out_stop_s > 0.0 {
            audio_ops::fade_stop_other_ids(&mut states, &args.id);
        } else {
            audio_ops::stop_other_ids(&mut states, &args.id);
        }
    }
    if audio_ops::should_skip_existing(&mut states, &args.id, args.overlap, args.restart) {
        return;
    }
    if let Some(source) =
        preload_cache::build_play_source(cache, &args.path, args.loop_mode, args.cue_start_s, args.cue_end_s)
    {
        let btn_state = bus.add_source(
            source, args.volume, args.duration, args.loop_mode, args.file_gain,
            args.fade_in_s, args.fade_out_stop_s, args.fade_out_end_s,
        );
        states.entry(args.id).or_default().push(btn_state);
    }
}

fn play_sequence(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&MasterBus>,
    id: String,
    paths: Vec<String>,
    volume: f32,
    duration: f64,
) {
    let Some(bus) = bus else { return; };
    let mut states = states.lock().unwrap();
    stop_removed(states.remove(&id));
    if let Some(seq) = SequenceSource::from_paths(&paths) {
        let btn_state = bus.add_source(Box::new(seq), volume, duration, false, 1.0, 0.0, 0.0, 0.0);
        states.entry(id).or_default().push(btn_state);
    }
}
