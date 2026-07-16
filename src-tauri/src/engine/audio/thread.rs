use crate::domain::playback::seek::{self as playback_seek, ReplayInfo};
use crate::engine::audio::button::ButtonStateMap;
use crate::engine::audio::command::AudioCommand;
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::audio::ops as audio_ops;
use crate::engine::audio::thread_play::{play_file, play_sequence, PlayArgs};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::{BusId, ConsoleEngine};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn spawn(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    cache: Arc<Mutex<PreloadCache>>,
    console: Arc<ConsoleEngine>,
) {
    thread::spawn(move || run(rx, states, last_pressed, cache, console));
}

fn run(
    rx: Receiver<AudioCommand>,
    states: Arc<Mutex<ButtonStateMap>>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    cache: Arc<Mutex<PreloadCache>>,
    console: Arc<ConsoleEngine>,
) {
    let mut replays: HashMap<String, ReplayInfo> = HashMap::new();
    // La ultima tarjeta pedida para el bus principal, para saber si de verdad
    // cambia. Reaplicar la misma salida (al arrancar, al reconectar) no debe
    // callar lo que este sonando.
    let mut current_main = String::new();

    for cmd in rx {
        audio_ops::purge_done(&states);
        match cmd {
            AudioCommand::SetDevice { device_name } => {
                // Cambiar de tarjeta se lleva por delante lo que sonaba: esas
                // fuentes viven en un bus que esta a punto de desaparecer, y sin
                // limpiar quedarian colgadas en el mapa sin llegar a terminar
                // nunca (nadie las itera ya, asi que jamas marcan done).
                // Que el bus no exista cuenta como cambio: hay que reabrirlo.
                if device_name != current_main || console.bus(BusId::Main).is_none() {
                    states.lock().unwrap().clear();
                    replays.clear();
                }
                current_main = device_name.clone();
                console.set_bus_device(BusId::Main, &device_name);
            }
            AudioCommand::SetPreDevice { device_name } => {
                // Sin tarjeta propia el bus de pre-escucha no existe, y quien
                // quiera sonar en el cae al principal. Ese fallback es el que la
                // Fase 3 elimina; aqui se conserva tal cual estaba.
                if device_name.is_empty() {
                    console.close_bus(BusId::Pre);
                } else {
                    console.set_bus_device(BusId::Pre, &device_name);
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
                group,
            } => {
                let main_button = !to_pre
                    && !id.starts_with("__")
                    && group == crate::engine::audio::button::PlaybackGroup::Main;
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
                    console.bus(BusId::Pre).or_else(|| console.bus(BusId::Main))
                } else {
                    console.bus(BusId::Main)
                };
                let played = play_file(
                    &states,
                    bus.as_ref(),
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
                        group,
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
            AudioCommand::StopGroupFade { group } => audio_ops::fade_stop_group(&states, group),
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
                    console.bus(BusId::Main).as_ref(),
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
                group,
            } => {
                play_sequence(
                    &states,
                    console.bus(BusId::Main).as_ref(),
                    id,
                    paths,
                    volume,
                    duration,
                    group,
                );
            }
        }
    }
}
