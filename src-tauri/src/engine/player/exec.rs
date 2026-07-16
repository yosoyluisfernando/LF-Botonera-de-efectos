//! Modulo: engine/player/exec.rs
//! Proposito: ejecutar sobre los decks las acciones que decide la cola. Es la
//! unica pieza que traduce `DeckAction` a operaciones de rodio; la cola decide y
//! aqui se obedece, sin logica de avance. Aqui se resuelven ademas los tipos
//! especiales, porque cargar es justo "el momento de sonar". Reutiliza
//! `build_play_source` (cue y cache) y `SequenceSource` (locuciones).
use super::deck::{Deck, DeckTrack};
use super::queue::DeckAction;
use super::resolve::{QueueResolver, ResolvedPlayback};
use crate::engine::audio::bus::SequenceSource;
use crate::engine::audio::decode::BoxSource;
use crate::engine::cache::preload::{build_play_source, PreloadCache};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

pub fn exec_all(
    actions: Vec<DeckAction>,
    decks: &mut [Deck],
    cache: &Arc<Mutex<PreloadCache>>,
    volume: &Arc<AtomicU32>,
    resolver: &QueueResolver,
) {
    for action in actions {
        exec(action, decks, cache, volume, resolver);
    }
}

/// Construye la fuente de una entrada ya resuelta. Una sola pista pasa por
/// `build_play_source` (respeta cue y cache); una locucion son varios archivos
/// que suenan seguidos, y para eso ya existe `SequenceSource`.
fn source_for(play: &ResolvedPlayback, cache: &Arc<Mutex<PreloadCache>>) -> Option<BoxSource> {
    match play.paths.as_slice() {
        [] => None,
        [one] => build_play_source(cache, one, false, play.cue_start_s, play.cue_end_s),
        many => SequenceSource::from_paths(many).map(|s| Box::new(s) as BoxSource),
    }
}

/// Lo que el deck debe recordar para poder recargarse en un seek. Solo una pista
/// de un archivo es reposicionable: una locucion va encadenada.
fn track_of(play: &ResolvedPlayback) -> DeckTrack {
    DeckTrack {
        path: play.paths.first().cloned().unwrap_or_default(),
        duration_s: play.duration_s,
        gain: play.gain,
        cue_start_s: play.cue_start_s,
        cue_end_s: play.cue_end_s,
        seekable: play.paths.len() == 1,
    }
}

fn exec(
    action: DeckAction,
    decks: &mut [Deck],
    cache: &Arc<Mutex<PreloadCache>>,
    volume: &Arc<AtomicU32>,
    resolver: &QueueResolver,
) {
    match action {
        DeckAction::Load { deck, entry, autoplay } => {
            let Some(target) = decks.get_mut(deck) else {
                return;
            };
            let vol = f32::from_bits(volume.load(Ordering::Relaxed));
            // Se resuelve AHORA: la hora avanza, el clima cambia y el aleatorio
            // debe dar una cancion nueva en cada pasada.
            let source = resolver
                .resolve(&entry)
                .and_then(|play| source_for(&play, cache).map(|s| (s, play)));
            let Some((source, play)) = source else {
                // Sin fuente (carpeta vacia, sin clima, archivo ilegible): se
                // marca fallido para que el motor releve y la musica siga.
                target.fail();
                return;
            };
            target.load(source, track_of(&play), vol);
            if autoplay {
                target.play();
            }
        }
        // Seek: se reconstruye la MISMA pista desde `position_s`. La ruta sale
        // del deck (ya resuelta), no de la cola: en una carpeta aleatoria la
        // cola solo sabe la carpeta, y recargar debe sonar la misma cancion.
        DeckAction::Seek { deck, position_s } => {
            let Some(target) = decks.get_mut(deck) else {
                return;
            };
            if !target.can_seek() {
                return;
            }
            let t = target.track().clone();
            let vol = f32::from_bits(volume.load(Ordering::Relaxed));
            let at = (t.cue_start_s + position_s).max(0.0);
            if let Some(source) = build_play_source(cache, &t.path, false, at, t.cue_end_s) {
                target.reload_at(source, position_s, vol);
            }
        }
        DeckAction::Resume { deck } => {
            if let Some(d) = decks.get_mut(deck) {
                d.play();
            }
        }
        DeckAction::Stop { deck } => {
            if let Some(d) = decks.get_mut(deck) {
                d.stop();
            }
        }
        DeckAction::StopAll => {
            for d in decks.iter_mut() {
                d.stop();
            }
        }
    }
}
