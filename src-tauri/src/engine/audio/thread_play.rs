use crate::domain::playback::source as playback_source;
use crate::engine::audio::attach::{attach_button, AttachArgs};
use crate::engine::audio::button::{ButtonStateMap, PlaybackGroup, ReplayInfo};
/// Modulo: audio_thread_play.rs
/// Proposito: operaciones auxiliares usadas por el hilo de audio.
use crate::engine::audio::ops::{self as audio_ops, stop_removed};
use crate::engine::audio::sequence::SequenceSource;
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::Bus;
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
    pub group: PlaybackGroup,
}

pub fn play_file(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&Bus>,
    cache: &Arc<Mutex<PreloadCache>>,
    args: PlayArgs,
) -> bool {
    let Some(bus) = bus else { return false };
    let mut states = states.lock().unwrap();
    if args.stop_other {
        if args.fade_out_stop_s > 0.0 {
            audio_ops::fade_stop_other_ids(&mut states, &args.id, args.group);
        } else {
            audio_ops::stop_other_ids(&mut states, &args.id, args.group);
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
    // La ficha para rehacer esta fuente si su bus muere (un cambio de tarjeta).
    // La lleva el estado, no un mapa por id: con `overlap` hay varias instancias
    // del mismo boton sonando por su sitio, y una ficha compartida las rehariat
    // odas en la misma posicion.
    let replay = Arc::new(ReplayInfo {
        id: args.id.clone(),
        path: args.path.clone(),
        volume: args.volume,
        duration: args.duration,
        loop_mode: args.loop_mode,
        cue_start_s: args.cue_start_s,
        cue_end_s: args.cue_end_s,
        file_gain: args.file_gain,
        fade_in_s: args.fade_in_s,
        fade_out_stop_s: args.fade_out_stop_s,
        fade_out_end_s: args.fade_out_end_s,
    });
    let btn_state = attach_button(
        bus,
        source,
        AttachArgs {
            volume: args.volume,
            duration: args.duration,
            loop_mode: args.loop_mode,
            file_gain: args.file_gain,
            fade_in_s: args.fade_in_s,
            fade_out_stop_s: args.fade_out_stop_s,
            fade_out_end_s: args.fade_out_end_s,
            position_offset_s: args.position_offset_s,
            group: args.group,
            replay: Some(replay),
        },
    );
    states.entry(args.id).or_default().push(btn_state);
    true
}

pub fn play_sequence(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&Bus>,
    id: String,
    paths: Vec<String>,
    volume: f32,
    duration: f64,
    group: PlaybackGroup,
) {
    let Some(bus) = bus else { return };
    let mut states = states.lock().unwrap();
    stop_removed(states.remove(&id));
    if let Some(seq) = SequenceSource::from_paths(&paths) {
        let btn_state = attach_button(
            bus,
            Box::new(seq),
            AttachArgs {
                volume,
                duration,
                loop_mode: false,
                file_gain: 1.0,
                fade_in_s: 0.0,
                fade_out_stop_s: 0.0,
                fade_out_end_s: 0.0,
                position_offset_s: 0.0,
                group,
                // Una locucion son varios archivos encadenados en una sola
                // fuente: no se puede reposicionar, asi que no se puede rehacer.
                // Si la tarjeta cambia a mitad, se cae y ya.
                replay: None,
            },
        );
        states.entry(id).or_default().push(btn_state);
    }
}
