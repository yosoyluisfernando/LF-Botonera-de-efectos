/// Módulo: audio_ops.rs
/// Propósito: operaciones sobre el mapa de estados de botones (purgar, parar,
/// volumen). Separadas del hilo de audio (audio_thread.rs) por cohesión: aquí
/// vive el "control de fuentes activas", allí la ejecución del motor.
use crate::master_bus::{ButtonState, ButtonStateMap};
use std::sync::{Arc, Mutex};

/// Elimina del mapa las fuentes ya terminadas.
pub fn purge_done(states: &Arc<Mutex<ButtonStateMap>>) {
    states.lock().unwrap().retain(|_, group| {
        group.retain(|state| !state.is_done());
        !group.is_empty()
    });
}

/// Para todas las fuentes salvo las del id dado (modo "detener otros").
pub fn stop_other_ids(states: &mut ButtonStateMap, id: &str) {
    states
        .iter()
        .filter(|(key, _)| key.as_str() != id)
        .flat_map(|(_, group)| group.iter())
        .for_each(|state| state.stop());
    states.retain(|key, _| key.as_str() == id);
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

pub fn stop_id(states: &Arc<Mutex<ButtonStateMap>>, id: &str) {
    stop_removed(states.lock().unwrap().remove(id));
}

pub fn stop_all(states: &Arc<Mutex<ButtonStateMap>>) {
    let mut states = states.lock().unwrap();
    for (_, group) in states.drain() {
        stop_removed(Some(group));
    }
}

pub fn set_volume(states: &Arc<Mutex<ButtonStateMap>>, id: &str, volume: f32) {
    if let Some(group) = states.lock().unwrap().get(id) {
        for state in group {
            state.set_volume(volume);
        }
    }
}

/// Marca como detenidas todas las fuentes de un grupo retirado del mapa.
pub fn stop_removed(group: Option<Vec<ButtonState>>) {
    if let Some(group) = group {
        for state in group {
            state.stop();
        }
    }
}
