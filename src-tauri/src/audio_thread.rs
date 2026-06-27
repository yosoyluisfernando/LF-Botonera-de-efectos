/// Modulo: audio_thread.rs
/// Proposito: ejecutar el motor de audio en un hilo dedicado.
use crate::audio_command::AudioCommand;
use crate::audio_device::AudioDeviceRuntime;
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

    for cmd in rx {
        purge_done(&states);
        match cmd {
            AudioCommand::SetDevice { device_name } => {
                device.set_device(&states, &master_l, &master_r, &master_volume, device_name);
            }
            AudioCommand::Play {
                id,
                path,
                volume,
                duration,
                loop_mode,
                stop_other,
                overlap,
                restart,
                cue_start_s,
                cue_end_s,
                file_gain,
            } => {
                play_file(
                    &states,
                    device.bus(),
                    &cache,
                    PlayArgs {
                        id,
                        path,
                        volume,
                        duration,
                        loop_mode,
                        stop_other,
                        overlap,
                        restart,
                        cue_start_s,
                        cue_end_s,
                        file_gain,
                    },
                );
            }
            AudioCommand::Stop { id } => stop_id(&states, &id),
            AudioCommand::StopAll => stop_all(&states),
            AudioCommand::SetVolume { id, volume } => set_volume(&states, &id, volume),
            AudioCommand::PlaySequence {
                id,
                paths,
                volume,
                duration,
            } => {
                play_sequence(&states, device.bus(), id, paths, volume, duration);
            }
        }
    }
}

fn purge_done(states: &Arc<Mutex<ButtonStateMap>>) {
    states.lock().unwrap().retain(|_, group| {
        group.retain(|state| !state.is_done());
        !group.is_empty()
    });
}

/// Parámetros de una reproducción (incluye cue y ganancia del archivo, E.d).
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
}

fn play_file(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&MasterBus>,
    cache: &Arc<Mutex<PreloadCache>>,
    args: PlayArgs,
) {
    let Some(bus) = bus else {
        return;
    };
    let mut states = states.lock().unwrap();
    if args.stop_other {
        stop_other_ids(&mut states, &args.id);
    }
    if should_skip_existing(&mut states, &args.id, args.overlap, args.restart) {
        return;
    }
    if let Some(source) =
        preload_cache::build_play_source(cache, &args.path, args.loop_mode, args.cue_start_s, args.cue_end_s)
    {
        let btn_state = bus.add_source(source, args.volume, args.duration, args.loop_mode, args.file_gain);
        states.entry(args.id).or_default().push(btn_state);
    }
}

fn stop_other_ids(states: &mut ButtonStateMap, id: &str) {
    states
        .iter()
        .filter(|(key, _)| key.as_str() != id)
        .flat_map(|(_, group)| group.iter())
        .for_each(|state| state.stop());
    states.retain(|key, _| key.as_str() == id);
}

fn should_skip_existing(states: &mut ButtonStateMap, id: &str, overlap: bool, restart: bool) -> bool {
    if !states.get(id).map_or(false, |group| !group.is_empty()) || overlap {
        return false;
    }
    stop_removed(states.remove(id));
    !restart
}

fn stop_id(states: &Arc<Mutex<ButtonStateMap>>, id: &str) {
    stop_removed(states.lock().unwrap().remove(id));
}

fn stop_all(states: &Arc<Mutex<ButtonStateMap>>) {
    let mut states = states.lock().unwrap();
    for (_, group) in states.drain() {
        stop_removed(Some(group));
    }
}

fn set_volume(states: &Arc<Mutex<ButtonStateMap>>, id: &str, volume: f32) {
    if let Some(group) = states.lock().unwrap().get(id) {
        for state in group {
            state.set_volume(volume);
        }
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
    let Some(bus) = bus else {
        return;
    };
    let mut states = states.lock().unwrap();
    stop_removed(states.remove(&id));
    if let Some(seq) = SequenceSource::from_paths(&paths) {
        let btn_state = bus.add_source(Box::new(seq), volume, duration, false, 1.0);
        states.entry(id).or_default().push(btn_state);
    }
}

fn stop_removed(group: Option<Vec<crate::master_bus::ButtonState>>) {
    if let Some(group) = group {
        for state in group {
            state.stop();
        }
    }
}
