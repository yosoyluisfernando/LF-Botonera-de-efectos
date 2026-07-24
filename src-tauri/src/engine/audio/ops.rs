/// Módulo: audio_ops.rs
/// Propósito: operaciones sobre el mapa de estados de botones (purgar, parar,
/// volumen). Separadas del hilo de audio (audio_thread.rs) por cohesión.
use crate::engine::audio::button::{ButtonState, ButtonStateMap, PlaybackGroup};
use std::sync::{Arc, Mutex};

/// Elimina del mapa las fuentes ya terminadas.
pub fn purge_done(states: &Arc<Mutex<ButtonStateMap>>) {
    states.lock().unwrap().retain(|_, group| {
        group.retain(|state| !state.is_done());
        !group.is_empty()
    });
}

/// Para otras fuentes con corte inmediato y elimina del mapa (sin fade).
pub fn stop_other_ids(states: &mut ButtonStateMap, id: &str, target: PlaybackGroup) {
    states
        .iter()
        .filter(|(key, group)| key.as_str() != id && group.iter().any(|s| s.group == target))
        .flat_map(|(_, group)| group.iter())
        .filter(|state| state.group == target)
        .for_each(|state| state.stop_immediate());
    states.retain(|key, group| key.as_str() == id || group.iter().any(|s| s.group != target));
}

/// Para otras fuentes con fundido si está configurado; las mantiene en el mapa
/// hasta que terminen (purge_done las retira cuando done_flag = true).
pub fn fade_stop_other_ids(states: &mut ButtonStateMap, id: &str, target: PlaybackGroup) {
    states
        .iter()
        .filter(|(key, _)| key.as_str() != id)
        .flat_map(|(_, group)| group.iter())
        .filter(|state| state.group == target)
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

pub fn fade_stop_group(states: &Arc<Mutex<ButtonStateMap>>, target: PlaybackGroup) {
    for group in states.lock().unwrap().values() {
        group
            .iter()
            .filter(|s| s.group == target)
            .for_each(ButtonState::stop);
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

#[cfg(test)]
mod tests {
    use super::{fade_stop_other_ids, stop_other_ids};
    use crate::engine::audio::button::{ButtonState, ButtonStateMap, PlaybackGroup};
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::Arc;
    use std::time::Instant;

    #[test]
    fn stop_other_isolated_by_playback_group() {
        let mut states = ButtonStateMap::new();
        states.insert("main".into(), vec![state(PlaybackGroup::Main)]);
        states.insert("cue".into(), vec![state(PlaybackGroup::Cue)]);
        states.insert("fixed-a".into(), vec![state(PlaybackGroup::Fixed)]);
        states.insert("fixed-b".into(), vec![state(PlaybackGroup::Fixed)]);
        stop_other_ids(&mut states, "fixed-a", PlaybackGroup::Fixed);
        assert!(states.contains_key("main"));
        assert!(states.contains_key("cue"));
        assert!(states.contains_key("fixed-a"));
        assert!(!states.contains_key("fixed-b"));
    }

    #[test]
    fn main_stop_other_does_not_stop_cue() {
        let mut states = ButtonStateMap::new();
        states.insert("main-a".into(), vec![state(PlaybackGroup::Main)]);
        states.insert("main-b".into(), vec![state(PlaybackGroup::Main)]);
        states.insert("cue".into(), vec![state(PlaybackGroup::Cue)]);
        stop_other_ids(&mut states, "main-b", PlaybackGroup::Main);
        assert!(!states.contains_key("main-a"));
        assert!(states.contains_key("main-b"));
        assert!(states.contains_key("cue"));
    }

    #[test]
    fn main_fade_stop_other_does_not_stop_cue() {
        let mut states = ButtonStateMap::new();
        states.insert("main".into(), vec![state(PlaybackGroup::Main)]);
        states.insert("cue".into(), vec![state(PlaybackGroup::Cue)]);
        fade_stop_other_ids(&mut states, "new-main", PlaybackGroup::Main);
        assert!(states["main"][0]
            .stop_flag
            .load(std::sync::atomic::Ordering::Relaxed));
        assert!(!states["cue"][0]
            .stop_flag
            .load(std::sync::atomic::Ordering::Relaxed));
    }

    fn state(group: PlaybackGroup) -> ButtonState {
        ButtonState {
            group,
            done_flag: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            fade_out_flag: None,
            volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            start_time: Instant::now(),
            position_offset_s: 0.0,
            duration: 1.0,
            loop_mode: false,
            replay: None,
        }
    }
}
