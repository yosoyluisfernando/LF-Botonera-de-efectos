//! Reconciliación incremental: conserva buses, fuentes y medidores ajenos al cambio.
use super::bus::{Bus, BusOutput};
use super::endpoint::EndpointRegistry;
use super::ConsoleState;
use crate::domain::console::routing::devices_in_use;
use crate::domain::console::{device_of, effective, BusId, Routing};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};

pub fn update(endpoints: &mut EndpointRegistry, state: &Arc<Mutex<ConsoleState>>, force: &[BusId]) -> bool {
    let mut guard = state.lock().unwrap();
    let program_device = super::device::canonical_name(&program_device(&guard));
    let targets: std::collections::HashMap<_, _> = BusId::every().filter_map(|bus| {
        let routing = effective_of(&guard, bus, &program_device)?;
        let target = if !bus.wants_output(&routing) { routing } else {
            device_of(&routing, &program_device).map(|d| Routing::Device(super::device::canonical_name(&d))).unwrap_or(Routing::Program)
        };
        Some((bus, target))
    }).collect();
    let changed = super::graph_update::apply(&mut guard, &targets, force, |bus, state| {
        if bus == BusId::Programa { open_program(endpoints, state, &program_device); }
        else { open_bus(endpoints, state, bus, &program_device); }
    });
    if !changed { return false; }
    let live = live_routings(&guard, &program_device);
    drop(guard);
    endpoints.retain_only(&devices_in_use(&live, &program_device));
    true
}

/// El ruteo que se aplica de verdad a un bus: el pedido, ya resuelto contra la
/// tarjeta del programa.
fn effective_of(state: &ConsoleState, bus: BusId, program_device: &str) -> Option<Routing> {
    let slot = state.slots.get(&bus)?;
    let routing = match &slot.routing {
        Routing::Device(name) if bus.wants_output(&slot.routing) => Routing::Device(super::device::canonical_name(name)),
        other => other.clone(),
    };
    Some(effective(bus, &routing, program_device))
}

/// El ruteo EFECTIVO de los buses que han quedado vivos, para saber que tarjetas
/// siguen haciendo falta. El efectivo y no el pedido: un bus que pidio la tarjeta
/// del programa acaba sumado en el, y entonces no retiene tarjeta propia.
fn live_routings(state: &ConsoleState, program_device: &str) -> Vec<(BusId, Routing)> {
    BusId::every()
        .filter(|bus| state.live.contains_key(bus))
        .filter_map(|bus| effective_of(state, bus, program_device).map(|r| (bus, r)))
        .collect()
}

/// La tarjeta del programa. `sanitize` garantiza que su ruteo sea `Device`.
fn program_device(state: &ConsoleState) -> String {
    match state.slots.get(&BusId::Programa).map(|s| &s.routing) {
        Some(Routing::Device(name)) => name.clone(),
        _ => String::new(),
    }
}

fn open_program(endpoints: &mut EndpointRegistry, state: &mut ConsoleState, device: &str) {
    let Some(handle) = endpoints.ensure(device) else {
        return;
    };
    let (level_l, level_r, gain) = atomics(state, BusId::Programa);
    if let Some(bus) = Bus::open_controlled(BusOutput::Endpoint(&handle), level_l, level_r, gain, None, None, Some(Arc::clone(&state.master_ramp))) {
        state.live.insert(BusId::Programa, bus);
    }
}

fn open_bus(
    endpoints: &mut EndpointRegistry,
    state: &mut ConsoleState,
    bus: BusId,
    program_device: &str,
) {
    let Some(routing) = effective_of(state, bus, program_device) else {
        return;
    };
    if !bus.wants_output(&routing) { return; }
    let (level_l, level_r, gain) = atomics(state, bus);
    let independent = matches!(bus, BusId::Salida(_));
    let master = independent.then(|| Arc::clone(&state.slots[&BusId::Programa].volume));
    let duck = (bus.can_sum_into_program() || independent).then(|| super::duck::DuckStage {
        control: Arc::clone(&state.slots[&bus].duck),
        parent_ramp: (routing == Routing::Program || independent)
            .then(|| Arc::clone(&state.master_ramp)),
        parent: (routing == Routing::Program || independent)
            .then(|| Arc::clone(&state.slots[&BusId::Programa].volume)),
    });
    let opened = match device_of(&routing, program_device) {
        // Sin tarjeta propia: va sumado dentro del programa. Si el programa no
        // existe (su tarjeta fallo), este bus tampoco: no hay donde entregar.
        None => {
            let parent = state
                .live
                .get(&BusId::Programa)
                .map(|p| p.controller().clone());
            parent.and_then(|ctrl| {
                Bus::open_with_duck(BusOutput::Bus(&ctrl), level_l, level_r, gain, duck)
            })
        }
        Some(device) => endpoints.ensure(&device).and_then(|handle| {
            Bus::open_controlled(BusOutput::Endpoint(&handle), level_l, level_r, gain, duck, master, independent.then(|| Arc::clone(&state.master_ramp)))
        }),
    };
    if let Some(opened) = opened {
        state.live.insert(bus, opened);
    }
}

/// Los atomicos del bus. Viven en el slot y sobreviven a la reconstruccion: el
/// monitor y el fader los tienen cogidos desde antes.
fn atomics(state: &ConsoleState, bus: BusId) -> (Arc<AtomicU32>, Arc<AtomicU32>, Arc<AtomicU32>) {
    let slot = &state.slots[&bus];
    (
        Arc::clone(&slot.level_l),
        Arc::clone(&slot.level_r),
        Arc::clone(&slot.volume),
    )
}
