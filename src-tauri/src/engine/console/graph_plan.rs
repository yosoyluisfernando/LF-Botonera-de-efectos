//! Solo se invalidan buses cuyo enchufe o padre cambió.
use super::{BusId, ConsoleState, Routing};
use std::collections::{HashMap, HashSet};
pub fn affected(
    state: &ConsoleState,
    targets: &HashMap<BusId, Routing>,
    force: &[BusId],
) -> HashSet<BusId> {
    let mut dirty: HashSet<_> = targets
        .iter()
        .filter_map(|(bus, target)| {
            (state.slots[bus].applied.as_ref() != Some(target)).then_some(*bus)
        })
        .chain(force.iter().copied())
        .collect();
    if dirty.contains(&BusId::Programa) {
        for (bus, target) in targets {
            if *target == Routing::Program || state.slots[bus].applied == Some(Routing::Program) {
                dirty.insert(*bus);
            }
        }
    }
    dirty
}
