/// Modulo: audio_thread.rs
/// Proposito: ejecutar el motor de audio en un hilo dedicado.
use crate::audio_command::AudioCommand;
use crate::audio_decode;
use crate::audio_device::AudioDeviceRuntime;
use crate::master_bus::{ButtonStateMap, MasterBus, SequenceSource};
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn spawn(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    master_l: Arc<AtomicU32>,
    master_r: Arc<AtomicU32>,
    master_volume: Arc<AtomicU32>,
) {
    thread::spawn(move || run(rx, states, master_l, master_r, master_volume));
}

fn run(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    master_l: Arc<AtomicU32>,
    master_r: Arc<AtomicU32>,
    master_volume: Arc<AtomicU32>,
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
            } => {
                play_file(
                    &states,
                    device.bus(),
                    id,
                    path,
                    volume,
                    duration,
                    loop_mode,
                    stop_other,
                    overlap,
                    restart,
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

fn play_file(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&MasterBus>,
    id: String,
    path: String,
    volume: f32,
    duration: f64,
    loop_mode: bool,
    stop_other: bool,
    overlap: bool,
    restart: bool,
) {
    let Some(bus) = bus else {
        return;
    };
    let mut states = states.lock().unwrap();
    if stop_other {
        stop_other_ids(&mut states, &id);
    }
    if should_skip_existing(&mut states, &id, overlap, restart) {
        return;
    }
    if let Some(source) = audio_decode::source_from_path(&path, loop_mode) {
        let btn_state = bus.add_source(source, volume, duration, loop_mode);
        states.entry(id).or_default().push(btn_state);
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

fn should_skip_existing(
    states: &mut ButtonStateMap,
    id: &str,
    overlap: bool,
    restart: bool,
) -> bool {
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
        let btn_state = bus.add_source(Box::new(seq), volume, duration, false);
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
