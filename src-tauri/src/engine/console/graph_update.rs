//! Retira y abre únicamente los buses invalidados. El resto conserva sus fuentes.
use super::{graph_plan, BusId, ConsoleState, Routing};
use std::collections::HashMap;
pub fn apply(
    state: &mut ConsoleState,
    targets: &HashMap<BusId, Routing>,
    force: &[BusId],
    mut open: impl FnMut(BusId, &mut ConsoleState),
) -> bool {
    let dirty = graph_plan::affected(state, targets, force);
    if dirty.is_empty() {
        return false;
    }
    for bus in &dirty {
        if let Some(old) = state.live.remove(bus) {
            old.close();
        }
        let slot = state.slots.get_mut(bus).unwrap();
        slot.level_l.store(0, std::sync::atomic::Ordering::Relaxed);
        slot.level_r.store(0, std::sync::atomic::Ordering::Relaxed);
        slot.applied = targets.get(bus).cloned();
        slot.revision = slot.revision.wrapping_add(1);
    }
    for bus in BusId::every().filter(|b| dirty.contains(b)) {
        open(bus, state);
    }
    state.generation = state.generation.wrapping_add(1);
    true
}
