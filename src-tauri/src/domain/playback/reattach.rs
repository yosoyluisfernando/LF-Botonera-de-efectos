//! Conserva las fuentes de buses intactos; migra solo los sonidos cuyo destino cambió.
use crate::domain::playback::source as playback_source;
use crate::engine::audio::attach::{attach_button, AttachArgs};
use crate::engine::audio::button::{ButtonState, ButtonStateMap};
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::ConsoleEngine;
use std::sync::{Arc, Mutex};

/// Rehace en sus buses nuevos todo lo que siguiera sonando. Lo que no se pueda
/// rehacer se retira del mapa: si se quedara, el reloj y el vumetro contarian un
/// audio que ya no existe, y nunca terminaria (nadie itera esa fuente, asi que
/// jamas marca `done`).
pub fn reattach_all(
    states: &Arc<Mutex<ButtonStateMap>>,
    console: &ConsoleEngine,
    cache: &Arc<Mutex<PreloadCache>>,
) {
    let mut map = states.lock().unwrap();
    let old: Vec<(String, Vec<ButtonState>)> = map.drain().collect();
    for (id, group) in old {
        for state in group {
            if state.is_done() {
                continue;
            }
            let target = console.bus(console.button_bus(&id, state.group == crate::engine::audio::button::PlaybackGroup::Cue, state.group));
            if state.output.as_ref().zip(target.as_ref()).is_some_and(|(old, now)| old.same_as(now)) {
                map.entry(id.clone()).or_default().push(state);
                continue;
            }
            state.stop_immediate();
            if let Some(fresh) = rebuild_one(&state, console, cache) {
                map.entry(id.clone()).or_default().push(fresh);
            }
        }
    }
}

/// Vuelve a crear UNA fuente en su bus, por donde iba. None si no se puede.
fn rebuild_one(
    state: &ButtonState,
    console: &ConsoleEngine,
    cache: &Arc<Mutex<PreloadCache>>,
) -> Option<ButtonState> {
    let replay = state.replay.clone()?;
    let bus = console.bus(console.button_bus(&replay.id, state.group == crate::engine::audio::button::PlaybackGroup::Cue, state.group))?;
    let at = resume_position(state);
    let source = playback_source::build_seek(
        cache,
        &replay.path,
        replay.loop_mode,
        replay.cue_start_s,
        at,
        replay.cue_end_s,
    )?;
    // La fuente vieja ya esta muerta con su bus, pero marcarla cuesta un atomico
    // y evita que un resto suene si el bus viejo aun no se ha soltado del todo.
    state.stop_immediate();
    Some(attach_button(
        &bus,
        source,
        AttachArgs {
            // El volumen en vivo, no el de la ficha: el operador pudo haberlo
            // movido despues de dispararlo (la barra de pre-escucha lo hace).
            volume: state.live_volume(),
            duration: replay.duration,
            loop_mode: replay.loop_mode,
            file_gain: replay.file_gain,
            // Sin fade de entrada: esto no es un disparo nuevo, es la misma
            // fuente que sigue. Un fundido aqui se oiria como un bache.
            fade_in_s: 0.0,
            fade_out_stop_s: replay.fade_out_stop_s,
            fade_out_end_s: replay.fade_out_end_s,
            position_offset_s: at,
            group: state.group,
            replay: Some(Arc::clone(&replay)),
        },
    ))
}

/// Por donde retomar. En bucle cuenta la vuelta actual, no el total: la fuente
/// nueva arranca dentro del archivo, y el archivo dura una vuelta.
fn resume_position(state: &ButtonState) -> f64 {
    let pos = state.position();
    if state.loop_mode && state.duration > 0.0 {
        return pos.rem_euclid(state.duration);
    }
    // Un pelo antes del final: pedir exactamente el final devolveria una fuente
    // vacia que se daria por terminada al instante.
    pos.clamp(0.0, (state.duration - 0.02).max(0.0))
}
