//! Modulo: engine/console/mod.rs
//! Proposito: la consola de audio. Es el motor que posee las salidas fisicas y
//! los buses; los motores de efectos y del reproductor son sus CLIENTES, no sus
//! pares: le piden un bus y le entregan fuentes.
//!
//! Por que es un motor propio, y no una pieza dentro de engine/audio/:
//! `OutputStream` no es Send (lleva un cpal::Stream dentro), asi que alguien
//! tiene que ser su dueno y quedarse quieto en un hilo. Cuando cada motor abria
//! su propia tarjeta habia DOS hilos duenos de salidas, y por eso no podia
//! existir un punto de suma comun. La consola centraliza esa propiedad en un
//! hilo guardian y reparte lo que si viaja: `OutputStreamHandle` (Clone + Send +
//! Sync) y el controller de cada bus (Arc, Send + Sync).
//!
//! La topologia:
//!
//! ```text
//!   Efectos ─┐
//!            ├─► Programa ─► fader (master) ─► medidor ─► tarjeta
//!   Panel ───┘
//!
//!   Cue ───────────────────► fader ─────────► medidor ─► tarjeta (la del
//!                                                        programa si no tiene
//!                                                        una propia, PERO sin
//!                                                        sumar en el)
//! ```
//!
//! Las reglas de que va donde viven en `domain/console/`; aqui solo se obedecen.
//! Guia: Documentacion/PLAN_CONSOLA_VIRTUAL.md.
pub mod bus;
pub mod device;
mod endpoint;
mod fader;
mod graph;
mod level;
mod thread;

pub use bus::Bus;
pub use crate::domain::console::{BusId, Routing};

use crate::domain::console::sanitize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, sync_channel, Sender, SyncSender};
use std::sync::{Arc, Mutex};

pub enum ConsoleCommand {
    SetBusRouting {
        bus: BusId,
        routing: Routing,
        /// Avisa cuando el grafo ya esta rehecho. Quien vaya a rehacer sus
        /// fuentes necesita esperar: entregarlas al bus viejo seria tirarlas.
        done: Option<SyncSender<()>>,
    },
}

/// Lo que un bus conserva aunque su tarjeta no exista. Los atomicos se crean una
/// vez y viven para siempre: el monitor, el fader y las fuentes que ya suenan los
/// tienen cogidos, y reconstruir el grafo no debe dejarlos apuntando a la nada.
pub struct BusSlot {
    pub routing: Routing,
    pub level_l: Arc<AtomicU32>,
    pub level_r: Arc<AtomicU32>,
    /// El fader del bus: una etapa por la que pasa toda su señal una vez. Se
    /// mueve mientras suena y afecta a lo que ya esta sonando.
    pub volume: Arc<AtomicU32>,
}

impl BusSlot {
    fn new(routing: Routing) -> Self {
        Self {
            routing,
            level_l: Arc::new(AtomicU32::new(0)),
            level_r: Arc::new(AtomicU32::new(0)),
            volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
        }
    }
}

#[derive(Default)]
pub struct ConsoleState {
    pub slots: HashMap<BusId, BusSlot>,
    /// Solo estan los buses con salida viva. Que un bus falte aqui significa
    /// exactamente "ese bus no existe ahora mismo".
    pub live: HashMap<BusId, Bus>,
    /// Sube cada vez que el grafo se rehace. Un motor que la vea cambiar sabe que
    /// las fuentes que habia entregado murieron con los buses viejos y que, si
    /// quiere seguir sonando, tiene que rehacerlas.
    pub generation: u64,
}

/// Handle de la consola (Send + Sync). No toca audio: los cambios de ruteo van
/// por el canal al hilo guardian. Leer un bus si es directo, porque un `Bus` es
/// solo Arcs y anadirle fuentes no requiere ser dueno de nada.
pub struct ConsoleEngine {
    tx: Sender<ConsoleCommand>,
    state: Arc<Mutex<ConsoleState>>,
}

impl ConsoleEngine {
    pub fn new() -> Self {
        let (tx, rx) = channel::<ConsoleCommand>();
        let slots = BusId::ALL
            .iter()
            .map(|bus| (*bus, BusSlot::new(bus.default_routing())))
            .collect();
        let state = Arc::new(Mutex::new(ConsoleState {
            slots,
            live: HashMap::new(),
            generation: 0,
        }));
        let state_thread = Arc::clone(&state);
        std::thread::spawn(move || thread::run(rx, state_thread));
        Self { tx, state }
    }

    /// Cambia a donde entrega un bus. Asincrono.
    pub fn set_bus_routing(&self, bus: BusId, routing: Routing) {
        let _ = self.tx.send(ConsoleCommand::SetBusRouting {
            bus,
            routing: sanitize(bus, routing),
            done: None,
        });
    }

    /// Igual, pero espera a que el grafo este rehecho.
    ///
    /// Lo usa quien va a devolver sus fuentes al bus nuevo justo despues:
    /// entregarlas antes seria darselas al bus viejo, que esta a punto de morir,
    /// y se perderian sin que nada avisara.
    pub fn set_bus_routing_sync(&self, bus: BusId, routing: Routing) {
        let (done, wait) = sync_channel(0);
        let sent = self.tx.send(ConsoleCommand::SetBusRouting {
            bus,
            routing: sanitize(bus, routing),
            done: Some(done),
        });
        if sent.is_ok() {
            // Con tope: si la consola se atasca abriendo una tarjeta que no
            // responde, mas vale seguir sin restaurar que colgar el motor.
            let _ = wait.recv_timeout(std::time::Duration::from_secs(2));
        }
    }

    /// Sube cada vez que el grafo se rehace. Comparala con la que viste la ultima
    /// vez para saber si las fuentes que entregaste siguen vivas.
    pub fn generation(&self) -> u64 {
        self.state.lock().unwrap().generation
    }

    /// El bus, si existe ahora mismo. Clonar es barato (solo Arcs) y suelta el
    /// candado en el acto: quien reproduce no debe esperar a la consola.
    pub fn bus(&self, bus: BusId) -> Option<Bus> {
        self.state.lock().unwrap().live.get(&bus).cloned()
    }

    /// Los niveles medidos del bus: (L, R). Existen aunque el bus no tenga
    /// salida, porque el monitor los tiene cogidos desde el arranque.
    pub fn levels(&self, bus: BusId) -> (Arc<AtomicU32>, Arc<AtomicU32>) {
        let state = self.state.lock().unwrap();
        let slot = &state.slots[&bus];
        (Arc::clone(&slot.level_l), Arc::clone(&slot.level_r))
    }

    /// A cuanto esta el fader del bus.
    pub fn fader(&self, bus: BusId) -> f32 {
        f32::from_bits(self.state.lock().unwrap().slots[&bus].volume.load(Ordering::Relaxed))
    }

    /// El atomico del fader, para quien lo lea muy a menudo (el monitor del
    /// reproductor, 10 veces por segundo) y no quiera pagar el candado cada vez.
    pub fn volume_handle(&self, bus: BusId) -> Arc<AtomicU32> {
        Arc::clone(&self.state.lock().unwrap().slots[&bus].volume)
    }

    /// Mueve el fader del bus. Afecta a lo que YA esta sonando por el.
    ///
    /// No pone techo: el maximo es una regla de producto (el modo boost llega a
    /// 1.5) y la decide quien llama. Aqui solo se impide el negativo, que no es
    /// "mas bajo" sino invertir la fase.
    pub fn set_fader(&self, bus: BusId, value: f32) {
        self.state.lock().unwrap().slots[&bus]
            .volume
            .store(value.max(0.0).to_bits(), Ordering::Relaxed);
    }
}

impl Default for ConsoleEngine {
    fn default() -> Self {
        Self::new()
    }
}
