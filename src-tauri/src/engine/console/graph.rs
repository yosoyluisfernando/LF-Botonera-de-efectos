//! Modulo: engine/console/graph.rs
//! Proposito: montar el grafo de buses de la consola segun el ruteo pedido.
//!
//! Se reconstruye ENTERO ante cualquier cambio, y no es pereza: rodio no sabe
//! sacar una fuente de un mixer. Remendar el grafo dejaria el mixer de un bus
//! viejo colgado dentro del programa para siempre, produciendo silencio y sin
//! que nadie pueda quitarlo. Reconstruir corta lo que suene —igual que ya hacia
//! cambiar de tarjeta— pero no acumula basura.
//!
//! El orden importa: el programa se abre PRIMERO, porque los buses que suman en
//! el se cuelgan de su controller.
use super::bus::{Bus, BusOutput};
use super::endpoint::EndpointRegistry;
use super::ConsoleState;
use crate::domain::console::routing::devices_in_use;
use crate::domain::console::{device_of, BusId, Routing};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};

pub fn rebuild(endpoints: &mut EndpointRegistry, state: &Arc<Mutex<ConsoleState>>) {
    let mut guard = state.lock().unwrap();
    guard.live.clear();

    let program_device = program_device(&guard);
    open_program(endpoints, &mut guard, &program_device);
    for bus in BusId::ALL {
        if bus != BusId::Programa {
            open_bus(endpoints, &mut guard, bus, &program_device);
        }
    }

    let live = live_routings(&guard);
    drop(guard);
    endpoints.retain_only(&devices_in_use(&live, &program_device));
}

/// El ruteo de los buses que han quedado vivos, para saber que tarjetas siguen
/// haciendo falta.
fn live_routings(state: &ConsoleState) -> Vec<(BusId, Routing)> {
    BusId::ALL
        .iter()
        .filter(|bus| state.live.contains_key(bus))
        .filter_map(|bus| state.slots.get(bus).map(|s| (*bus, s.routing.clone())))
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
    if let Some(bus) = Bus::open(BusOutput::Endpoint(&handle), level_l, level_r, gain) {
        state.live.insert(BusId::Programa, bus);
    }
}

fn open_bus(
    endpoints: &mut EndpointRegistry,
    state: &mut ConsoleState,
    bus: BusId,
    program_device: &str,
) {
    let Some(routing) = state.slots.get(&bus).map(|s| s.routing.clone()) else {
        return;
    };
    let (level_l, level_r, gain) = atomics(state, bus);
    let opened = match device_of(&routing, program_device) {
        // Sin tarjeta propia: va sumado dentro del programa. Si el programa no
        // existe (su tarjeta fallo), este bus tampoco: no hay donde entregar.
        None => {
            let parent = state.live.get(&BusId::Programa).map(|p| p.controller().clone());
            parent.and_then(|ctrl| Bus::open(BusOutput::Bus(&ctrl), level_l, level_r, gain))
        }
        Some(device) => endpoints
            .ensure(&device)
            .and_then(|handle| Bus::open(BusOutput::Endpoint(&handle), level_l, level_r, gain)),
    };
    if let Some(opened) = opened {
        state.live.insert(bus, opened);
    }
}

/// Los atomicos del bus. Viven en el slot y sobreviven a la reconstruccion: el
/// monitor y el fader los tienen cogidos desde antes.
fn atomics(
    state: &ConsoleState,
    bus: BusId,
) -> (Arc<AtomicU32>, Arc<AtomicU32>, Arc<AtomicU32>) {
    let slot = &state.slots[&bus];
    (
        Arc::clone(&slot.level_l),
        Arc::clone(&slot.level_r),
        Arc::clone(&slot.volume),
    )
}

