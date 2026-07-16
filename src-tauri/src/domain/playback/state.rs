/// Modulo: playback_state.rs
/// Proposito: mantener la identidad de reproduccion cuando una accion de grilla
/// cambia ids de botones sin detener el audio activo.
use crate::engine::audio::button::ButtonStateMap;
use crate::engine::audio::last_pressed::LastPressedInfo;
use std::sync::{Arc, Mutex};

/// Remapea estados activos desde ids antiguos hacia ids nuevos.
///
/// Se extraen todas las entradas antes de reinsertarlas para que un intercambio
/// A <-> B no sobrescriba accidentalmente ninguno de los dos estados.
pub fn remap_button_ids(
    button_states: Arc<Mutex<ButtonStateMap>>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    mappings: &[(String, String)],
) {
    if mappings.is_empty() {
        return;
    }

    remap_button_state_keys(&mut button_states.lock().unwrap(), mappings);
    remap_last_pressed(&mut last_pressed.lock().unwrap(), mappings);
}

fn remap_button_state_keys(map: &mut ButtonStateMap, mappings: &[(String, String)]) {
    let mut moved = Vec::new();
    for (old_id, new_id) in mappings {
        if old_id == new_id {
            continue;
        }
        if let Some(states) = map.remove(old_id) {
            moved.push((new_id.clone(), states));
        }
    }

    for (new_id, states) in moved {
        map.entry(new_id).or_default().extend(states);
    }
}

fn remap_last_pressed(last_pressed: &mut Option<LastPressedInfo>, mappings: &[(String, String)]) {
    let Some(info) = last_pressed else {
        return;
    };
    if let Some((_, new_id)) = mappings.iter().find(|(old_id, _)| old_id == &info.id) {
        info.id = new_id.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::remap_button_state_keys;
    use crate::engine::audio::button::ButtonStateMap;

    #[test]
    fn remaps_swapped_keys_without_losing_state_entries() {
        let mut map = ButtonStateMap::new();
        map.insert("a".to_string(), Vec::new());
        map.insert("b".to_string(), Vec::new());

        remap_button_state_keys(
            &mut map,
            &[
                ("a".to_string(), "b".to_string()),
                ("b".to_string(), "a".to_string()),
            ],
        );

        assert!(map.contains_key("a"));
        assert!(map.contains_key("b"));
        assert_eq!(map.len(), 2);
    }
}
