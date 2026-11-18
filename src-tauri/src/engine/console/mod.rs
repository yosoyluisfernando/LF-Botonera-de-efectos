//! La consola posee endpoints y buses; los motores entregan fuentes a sus handles.
pub mod bus;
pub mod controls;
pub mod device;
mod device_catalog;
#[cfg(any(target_os = "linux", test))]
mod device_linux;
mod device_migration;
#[cfg(windows)]
mod device_windows;
pub mod duck;
mod endpoint;
mod fader;
mod graph;
mod graph_plan;
mod graph_update;
mod handles;
mod level;
mod master_ramp;
pub mod tab_outputs;
mod tab_routes;
mod thread;
#[cfg(windows)]
mod wasapi_priority;
use crate::domain::console::sanitize;
pub use crate::domain::console::{BusId, Routing};
pub use bus::Bus;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{channel, sync_channel, Sender, SyncSender};
use std::sync::{Arc, Mutex};
pub enum ConsoleCommand {
    TabRoutes {
        plan: crate::domain::console::tab_lanes::LanePlan,
        available: Vec<String>,
        retry: bool,
        done: SyncSender<()>,
    },
    SetBusRouting {
        bus: BusId,
        routing: Routing,
        done: Option<SyncSender<()>>,
    },
}
pub struct BusSlot {
    pub routing: Routing,
    /// Último destino intentado; no implica que la tarjeta haya abierto.
    pub applied: Option<Routing>,
    pub revision: u64,
    pub level_l: Arc<AtomicU32>,
    pub level_r: Arc<AtomicU32>,
    pub volume: Arc<AtomicU32>,
    pub duck: Arc<duck::DuckControl>,
}
impl BusSlot {
    fn new(routing: Routing) -> Self {
        Self {
            routing,
            applied: None,
            revision: 0,
            level_l: Arc::new(AtomicU32::new(0)),
            level_r: Arc::new(AtomicU32::new(0)),
            volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            duck: Arc::new(duck::DuckControl::default()),
        }
    }
}
#[derive(Default)]
pub struct ConsoleState {
    pub master_ramp: Arc<master_ramp::MasterRamp>,
    pub slots: HashMap<BusId, BusSlot>,
    pub live: HashMap<BusId, Bus>,
    /// Notifica cambios de buses o asignaciones; cada cliente conserva fuentes intactas.
    pub generation: u64,
    pub tabs: HashMap<String, BusId>,
    pub available: Vec<String>,
}
pub struct ConsoleEngine {
    tx: Sender<ConsoleCommand>,
    state: Arc<Mutex<ConsoleState>>,
}
impl ConsoleEngine {
    pub fn new() -> Self {
        let (tx, rx) = channel::<ConsoleCommand>();
        let slots = BusId::every()
            .map(|bus| (bus, BusSlot::new(sanitize(bus, bus.default_routing()))))
            .collect();
        let state = Arc::new(Mutex::new(ConsoleState {
            slots,
            ..Default::default()
        }));
        let state_thread = Arc::clone(&state);
        std::thread::spawn(move || thread::run(rx, state_thread));
        Self { tx, state }
    }
    pub fn set_bus_routing(&self, bus: BusId, routing: Routing) {
        let _ = self.tx.send(ConsoleCommand::SetBusRouting {
            bus,
            routing: sanitize(bus, routing),
            done: None,
        });
    }
    pub fn set_bus_routing_sync(&self, bus: BusId, routing: Routing) {
        let (done, wait) = sync_channel(1);
        if self
            .tx
            .send(ConsoleCommand::SetBusRouting {
                bus,
                routing: sanitize(bus, routing),
                done: Some(done),
            })
            .is_ok()
        {
            let _ = wait.recv_timeout(std::time::Duration::from_secs(2));
        }
    }
}
impl Default for ConsoleEngine {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tab_output_tests;

#[cfg(test)]
mod continuity_tests;

#[cfg(test)]
mod volume_tests;
