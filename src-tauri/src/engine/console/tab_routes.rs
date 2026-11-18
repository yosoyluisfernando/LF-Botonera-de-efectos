//! Aplica rutas por lote y conserva los buses que no cambian.
use super::{
    endpoint::EndpointRegistry, graph, BusId, ConsoleCommand, ConsoleEngine, ConsoleState, Routing,
};
use crate::domain::console::tab_lanes::LanePlan;
use std::collections::HashMap;
use std::sync::{mpsc::sync_channel, Arc, Mutex};

impl ConsoleEngine {
    pub fn tab_lanes(&self) -> HashMap<String, u8> {
        self.state
            .lock()
            .unwrap()
            .slots
            .iter()
            .filter_map(|(bus, slot)| match (bus, &slot.routing) {
                (BusId::Salida(lane), Routing::Device(name)) if !name.is_empty() => {
                    Some((name.clone(), *lane))
                }
                _ => None,
            })
            .collect()
    }
    pub fn button_bus(
        &self,
        id: &str,
        pre: bool,
        group: crate::engine::audio::button::PlaybackGroup,
    ) -> BusId {
        let state = self.state.lock().unwrap();
        crate::engine::audio::routing::bus_for_button(&state.tabs, id, pre, group)
    }
    pub fn apply_tab_routes(
        &self,
        plan: LanePlan,
        available: Vec<String>,
        retry: bool,
    ) -> Result<(), String> {
        let (done, wait) = sync_channel(1);
        self.tx
            .send(ConsoleCommand::TabRoutes {
                plan,
                available,
                retry,
                done,
            })
            .map_err(|e| e.to_string())?;
        wait.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "tab_output_timeout".to_string())
    }
}

pub fn apply(
    plan: LanePlan,
    available: Vec<String>,
    retry: bool,
    endpoints: &mut EndpointRegistry,
    state: &Arc<Mutex<ConsoleState>>,
) {
    let mut guard = state.lock().unwrap();
    let mapping_changed = guard.tabs != plan.paleta_bus;
    let force: Vec<_> = guard.slots.iter().filter_map(|(bus, slot)| {
        let disappeared = match &slot.applied {
            Some(Routing::Device(name)) if !name.is_empty() => guard.available.contains(name) != available.contains(name),
            _ => false,
        };
        (disappeared || (retry && matches!(bus, BusId::Salida(_)) && !guard.live.contains_key(bus)
            && slot.routing != Routing::Device(String::new()))).then_some(*bus)
    }).collect();
    guard.tabs = plan.paleta_bus;
    guard.available = available.clone();
    for (bus, route) in plan.routings { guard.slots.get_mut(&bus).unwrap().routing = route; }
    drop(guard);
    endpoints.discard_missing(&available);
    let changed = graph::update(endpoints, state, &force);
    if mapping_changed && !changed {
        let mut guard = state.lock().unwrap();
        guard.generation = guard.generation.wrapping_add(1);
    }
}
