use crate::domain::playback::seek::{self as playback_seek, ReplayInfo};
use crate::engine::audio::button::ButtonStateMap;
use crate::engine::audio::command::AudioCommand;
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::audio::ops as audio_ops;
use crate::engine::audio::thread_play::{play_file, play_sequence, PlayArgs};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::{BusId, ConsoleEngine, Routing};
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
    // El ultimo ruteo pedido para el programa, para saber si de verdad cambia.
    // Reaplicar la misma salida (al arrancar, al reconectar) no debe callar lo
    // que este sonando.
    let mut current_program: Option<Routing> = None;

    for cmd in rx {
        audio_ops::purge_done(&states);
        match cmd {
            AudioCommand::SetBusRouting { bus, routing } => {
                // Cambiar el ruteo reconstruye el grafo, asi que se lleva por
                // delante lo que sonaba: esas fuentes viven en buses que estan a
                // punto de desaparecer, y sin limpiar quedarian colgadas en el
                // mapa sin terminar nunca (nadie las itera ya, asi que jamas
                // marcan done). Solo hace falta cuando el programa se mueve: es
                // el que arrastra a todos los buses que suman en el.
                if bus == BusId::Programa {
                    let changed = current_program.as_ref() != Some(&routing)
                        || console.bus(BusId::Programa).is_none();
                    if changed {
                        states.lock().unwrap().clear();
                        replays.clear();
                    }
                    current_program = Some(routing.clone());
                }
                console.set_bus_routing(bus, routing);
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
                bus,
                fade_in_s,
                fade_out_stop_s,
                fade_out_end_s,
                group,
            } => {
                let main_button = bus != BusId::Cue
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
                // Sin fallback: cada bus existe por su cuenta. La pre-escucha ya
                // no puede acabar en el programa por no tener tarjeta propia —
                // comparte el conector, no el bus.
                let played = play_file(
                    &states,
                    console.bus(bus).as_ref(),
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
                // El salto solo alcanza a los botones de la botonera principal
                // (`last_pressed`), asi que reconstruye en el bus de efectos.
                playback_seek::seek_active(
                    &states,
                    console.bus(BusId::Efectos).as_ref(),
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
                bus,
                group,
            } => {
                play_sequence(
                    &states,
                    console.bus(bus).as_ref(),
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
