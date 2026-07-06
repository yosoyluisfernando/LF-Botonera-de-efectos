/// Módulo: audio_ops.rs
/// Propósito: operaciones sobre el mapa de estados de botones (purgar, parar,
/// volumen). Separadas del hilo de audio (audio_thread.rs) por cohesión.
use crate::engine::audio::bus::{ButtonState, ButtonStateMap};
use std::sync::{Arc, Mutex};

/// Elimina del mapa las fuentes ya terminadas.
pub fn purge_done(states: &Arc<Mutex<ButtonStateMap>>) {
    states.lock().unwrap().retain(|_, group| {
        group.retain(|state| !state.is_done());
        !group.is_empty()
    });
}

/// Para otras fuentes con corte inmediato y elimina del mapa (sin fade).
pub fn stop_other_ids(states: &mut ButtonStateMap, id: &str) {
    states
        .iter()
        .filter(|(key, _)| key.as_str() != id)
        .flat_map(|(_, group)| group.iter())
        .for_each(|state| state.stop_immediate());
    states.retain(|key, _| key.as_str() == id);
}

/// Para otras fuentes con fundido si está configurado; las mantiene en el mapa
/// hasta que terminen (purge_done las retira cuando done_flag = true).
pub fn fade_stop_other_ids(states: &mut ButtonStateMap, id: &str) {
    states
        .iter()
        .filter(|(key, _)| key.as_str() != id)
        .flat_map(|(_, group)| group.iter())
        .for_each(|state| state.stop());
    // Sin retain: purge_done() limpiará cuando el fade termine.
}

/// ¿Saltar porque ya suena este id y no hay overlap? Aplica la regla de restart.
pub fn should_skip_existing(
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

/// Para el id dado con corte inmediato y lo elimina del mapa.
pub fn stop_id(states: &Arc<Mutex<ButtonStateMap>>, id: &str) {
    stop_removed(states.lock().unwrap().remove(id));
}

/// Para el id dado con fundido si está configurado; se mantiene en mapa.
pub fn fade_stop_id(states: &Arc<Mutex<ButtonStateMap>>, id: &str) {
    if let Some(group) = states.lock().unwrap().get(id) {
        for state in group {
            state.stop();
        }
    }
    // No se elimina del mapa: purge_done() lo retirará cuando done_flag = true.
}

pub fn stop_all(states: &Arc<Mutex<ButtonStateMap>>) {
    let mut states = states.lock().unwrap();
    for (_, group) in states.drain() {
        stop_removed(Some(group));
    }
}

/// Para todos con fundido si está configurado; los mantiene en el mapa.
pub fn fade_stop_all(states: &Arc<Mutex<ButtonStateMap>>) {
    for (_, group) in states.lock().unwrap().iter() {
        for state in group {
            state.stop();
        }
    }
}

pub fn set_volume(states: &Arc<Mutex<ButtonStateMap>>, id: &str, volume: f32) {
    if let Some(group) = states.lock().unwrap().get(id) {
        for state in group {
            state.set_volume(volume);
        }
    }
}

/// Marca como detenidas con corte inmediato (limpieza interna de grupos retirados).
pub fn stop_removed(group: Option<Vec<ButtonState>>) {
    if let Some(group) = group {
        for state in group {
            state.stop_immediate();
        }
    }
}
