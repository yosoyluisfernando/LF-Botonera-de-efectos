use crate::domain::playback::reattach;
use crate::domain::playback::seek as playback_seek;
use crate::engine::audio::button::ButtonStateMap;
use crate::engine::audio::command::AudioCommand;
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::audio::ops as audio_ops;
use crate::engine::audio::thread_play::{play_file, play_sequence, PlayArgs};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::{BusId, ConsoleEngine};
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
    for cmd in rx {
        audio_ops::purge_done(&states);
        match cmd {
            AudioCommand::SetBusRouting { bus, routing } => {
                // Cambiar de salida NO calla la botonera. Rehacer el grafo mata
                // las fuentes —rodio no sabe sacarlas de un mixer para llevarlas
                // a otro—, asi que se vuelven a crear en el segundo por el que
                // iban. Sincrono a proposito: entregarlas antes de que el grafo
                // este rehecho seria darselas al bus viejo, que ya esta muerto.
                //
                // Reaplicar el mismo ruteo no rehace nada: eso lo comprueba la
                // consola, que es quien sabe si el bus sigue vivo.
                console.set_bus_routing_sync(bus, routing);
                reattach::reattach_all(&states, &console, &cache);
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
                // Sin fallback: cada bus existe por su cuenta. La pre-escucha ya
                // no puede acabar en el programa por no tener tarjeta propia —
                // comparte el conector, no el bus.
                play_file(
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
            }
            AudioCommand::Stop { id } => audio_ops::stop_id(&states, &id),
            AudioCommand::StopFade { id } => audio_ops::fade_stop_id(&states, &id),
            AudioCommand::StopAll => audio_ops::stop_all(&states),
            AudioCommand::StopGroupFade { group } => audio_ops::fade_stop_group(&states, group),
            AudioCommand::StopAllFade => audio_ops::fade_stop_all(&states),
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
