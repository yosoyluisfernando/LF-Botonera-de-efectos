use crate::engine::audio::command::AudioCommand;
use crate::engine::audio::device::AudioDeviceRuntime;
use crate::engine::audio::ops as audio_ops;
use crate::engine::audio::thread_play::{play_file, play_sequence, PlayArgs};
use crate::engine::audio::bus::ButtonStateMap;
use crate::playback_seek::{self, ReplayInfo};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::audio::vu::LastPressedInfo;
use std::collections::HashMap;
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
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    cache: Arc<Mutex<PreloadCache>>,
) {
    thread::spawn(move || {
        run(
            rx,
            states,
            master_l,
            master_r,
            master_volume,
            last_pressed,
            cache,
        )
    });
}

fn run(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    master_l: Arc<AtomicU32>,
    master_r: Arc<AtomicU32>,
    master_volume: Arc<AtomicU32>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    cache: Arc<Mutex<PreloadCache>>,
) {
    let mut device = AudioDeviceRuntime::new();
    let mut device_pre = AudioDeviceRuntime::new();
    let pre_volume = Arc::new(AtomicU32::new(1.0f32.to_bits()));
    let pre_l = Arc::new(AtomicU32::new(0));
    let pre_r = Arc::new(AtomicU32::new(0));
    let mut replays: HashMap<String, ReplayInfo> = HashMap::new();

    for cmd in rx {
        audio_ops::purge_done(&states);
        match cmd {
            AudioCommand::SetDevice { device_name } => {
                device.set_device(
                    &states,
                    &master_l,
                    &master_r,
                    &master_volume,
                    device_name,
                    true,
                );
            }
            AudioCommand::SetPreDevice { device_name } => {
                if device_name.is_empty() {
                    device_pre.clear();
                } else {
                    device_pre.set_device(&states, &pre_l, &pre_r, &pre_volume, device_name, false);
                }
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
                to_pre,
                fade_in_s,
                fade_out_stop_s,
                fade_out_end_s,
            } => {
                let main_button = !to_pre && !id.starts_with("__");
                let replay = main_button.then(|| ReplayInfo {
                    id: id.clone(),
                    path: path.clone(),
                    volume,
                    duration,
                    loop_mode,
                    cue_start_s,
                    cue_end_s,
                    file_gain,
                    fade_in_s,
                    fade_out_stop_s,
                    fade_out_end_s,
                });
                if main_button && stop_other {
                    replays.retain(|key, _| key == &id);
                }
                let bus = if to_pre {
                    device_pre.bus().or_else(|| device.bus())
                } else {
                    device.bus()
                };
                let played = play_file(
                    &states,
                    bus,
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
                        fade_in_s,
                        fade_out_stop_s,
                        fade_out_end_s,
                        position_offset_s: 0.0,
                    },
                );
                if played {
                    if let Some(info) = replay {
                        replays.insert(info.id.clone(), info);
                    }
                }
            }
            AudioCommand::Stop { id } => {
                replays.remove(&id);
                audio_ops::stop_id(&states, &id)
            }
            AudioCommand::StopFade { id } => {
                replays.remove(&id);
                audio_ops::fade_stop_id(&states, &id)
            }
            AudioCommand::StopAll => {
                replays.clear();
                audio_ops::stop_all(&states)
            }
            AudioCommand::StopAllFade => {
                replays.clear();
                audio_ops::fade_stop_all(&states)
            }
            AudioCommand::SetVolume { id, volume } => audio_ops::set_volume(&states, &id, volume),
            AudioCommand::SeekActive {
                delta_s,
                position_s,
            } => {
                playback_seek::seek_active(
                    &states,
                    device.bus(),
                    &cache,
                    &last_pressed,
                    &replays,
                    delta_s,
                    position_s,
                );
            }
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
