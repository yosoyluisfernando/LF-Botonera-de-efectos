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
use crate::domain::console::{device_of, effective, BusId, Routing};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};

pub fn rebuild(endpoints: &mut EndpointRegistry, state: &Arc<Mutex<ConsoleState>>) {
    let mut guard = state.lock().unwrap();
    // Cerrarlos ANTES de soltarlos, y no solo soltarlos: a un mixer de rodio no
    // se le quita una fuente, asi que un bus soltado seguiria dentro de su
    // tarjeta sonando y midiendo. Y como los atomicos del medidor son del slot y
    // sobreviven, el viejo y el nuevo escribirian el mismo nivel a la vez —el
    // viejo, ya sin fuentes, escribiendo cero— y el vumetro parpadearia.
    for bus in guard.live.values() {
        bus.close();
    }
    guard.live.clear();
    // Los buses viejos acaban de morir, y con ellos las fuentes que tenian
    // dentro. Subir la generacion es como se enteran los motores de que lo que
    // estaban tocando ya no existe y hay que rehacerlo.
    guard.generation = guard.generation.wrapping_add(1);

    let program_device = program_device(&guard);
    open_program(endpoints, &mut guard, &program_device);
    for bus in BusId::ALL {
        if bus != BusId::Programa {
            open_bus(endpoints, &mut guard, bus, &program_device);
        }
    }

    let live = live_routings(&guard, &program_device);
    drop(guard);
    endpoints.retain_only(&devices_in_use(&live, &program_device));
}

/// El ruteo que se aplica de verdad a un bus: el pedido, ya resuelto contra la
/// tarjeta del programa.
fn effective_of(state: &ConsoleState, bus: BusId, program_device: &str) -> Option<Routing> {
    let slot = state.slots.get(&bus)?;
    Some(effective(bus, &slot.routing, program_device))
}

/// El ruteo EFECTIVO de los buses que han quedado vivos, para saber que tarjetas
/// siguen haciendo falta. El efectivo y no el pedido: un bus que pidio la tarjeta
/// del programa acaba sumado en el, y entonces no retiene tarjeta propia.
fn live_routings(state: &ConsoleState, program_device: &str) -> Vec<(BusId, Routing)> {
    BusId::ALL
        .iter()
        .filter(|bus| state.live.contains_key(bus))
        .filter_map(|bus| effective_of(state, *bus, program_device).map(|r| (*bus, r)))
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
    let Some(routing) = effective_of(state, bus, program_device) else {
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

