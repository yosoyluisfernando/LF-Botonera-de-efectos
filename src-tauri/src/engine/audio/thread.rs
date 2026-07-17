use crate::domain::playback::reattach;
use crate::domain::playback::seek as playback_seek;
use crate::engine::audio::button::ButtonStateMap;
use crate::engine::audio::command::AudioCommand;
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::audio::ops as audio_ops;
use crate::engine::audio::thread_play::{play_file, play_sequence, PlayArgs};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::{BusId, ConsoleEngine};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Cada cuánto se mira si el grafo cambió por decisión de otro motor. No es un
/// pulso de trabajo: el hilo sigue despertando al instante con cada comando.
const TICK: Duration = Duration::from_millis(100);

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
    let mut generation = console.generation();
    loop {
        // Con tope, y no bloqueado del todo: el grafo puede rehacerse por
        // decisión de OTRO motor —cambiar la salida del reproductor lo rehace
        // entero— y por aquí no pasaría ningún comando que lo delatara. Sin
        // mirarlo cada poco, este hilo se enteraría solo de sus propios cambios.
        let cmd = match rx.recv_timeout(TICK) {
            Ok(cmd) => Some(cmd),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if let Some(cmd) = cmd {
            audio_ops::purge_done(&states);
            handle(cmd, &states, &last_pressed, &cache, &console);
        }
        // ¿Se rehizo el grafo? Entonces lo que sonaba murió con su bus, lo haya
        // pedido quien lo haya pedido. Se rehace por donde iba.
        let now = console.generation();
        if now != generation {
            generation = now;
            reattach::reattach_all(&states, &console, &cache);
        }
    }
}

fn handle(
    cmd: AudioCommand,
    states: &Arc<Mutex<ButtonStateMap>>,
    last_pressed: &Arc<Mutex<Option<LastPressedInfo>>>,
    cache: &Arc<Mutex<PreloadCache>>,
    console: &Arc<ConsoleEngine>,
) {
    match cmd {
        AudioCommand::SetBusRouting { bus, routing } => {
            // Síncrono a propósito: el bucle rehace las fuentes en cuanto ve
            // subir la generación, y entregarlas antes de que el grafo esté
            // montado sería dárselas al bus viejo, que ya está muerto.
            //
            // Reaplicar el mismo ruteo no rehace nada: eso lo comprueba la
            // consola, que es quien sabe si el bus sigue vivo.
            console.set_bus_routing_sync(bus, routing);
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
