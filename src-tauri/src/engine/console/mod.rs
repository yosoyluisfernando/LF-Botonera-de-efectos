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
//! Guia: Documentacion/PLAN_CONSOLA_VIRTUAL.md.
pub mod bus;
pub mod device;
mod endpoint;
mod level;
mod thread;

pub use bus::Bus;

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

/// Los buses de la consola. Hoy son los dos que ya existian de hecho; la Fase 3
/// los abre a Efectos / Panel / Reproductor / Cue.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BusId {
    /// Salida principal: efectos de la botonera y del panel fijo.
    Main,
    /// Pre-escucha. Si no tiene tarjeta propia, hoy no existe y quien quiera
    /// sonar cae al bus Main — el fallback que la Fase 3 elimina.
    Pre,
}

pub enum ConsoleCommand {
    SetBusDevice { bus: BusId, device_name: String },
    CloseBus { bus: BusId },
}

/// Lo que un bus conserva aunque su tarjeta no exista. Los atomicos se crean una
/// vez y viven para siempre: el monitor y las fuentes que ya suenan los tienen
/// cogidos, y un cambio de tarjeta no debe dejarlos apuntando a la nada.
pub struct BusSlot {
    pub device_name: String,
    pub level_l: Arc<AtomicU32>,
    pub level_r: Arc<AtomicU32>,
    /// Volumen del bus. Hoy lo aplica cada fuente por su cuenta; la Fase 2 lo
    /// convierte en el fader real del bus.
    pub volume: Arc<AtomicU32>,
}

impl BusSlot {
    fn new(volume: f32) -> Self {
        Self {
            device_name: String::new(),
            level_l: Arc::new(AtomicU32::new(0)),
            level_r: Arc::new(AtomicU32::new(0)),
            volume: Arc::new(AtomicU32::new(volume.to_bits())),
        }
    }
}

#[derive(Default)]
pub struct ConsoleState {
    pub slots: HashMap<BusId, BusSlot>,
    /// Solo estan los buses con tarjeta viva. Que un bus falte aqui significa
    /// exactamente "ese bus no existe ahora mismo".
    pub live: HashMap<BusId, Bus>,
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
        let mut slots = HashMap::new();
        slots.insert(BusId::Main, BusSlot::new(1.0));
        // La pre-escucha no obedece al master: suena a lo que pida el operador.
        slots.insert(BusId::Pre, BusSlot::new(1.0));
        let state = Arc::new(Mutex::new(ConsoleState {
            slots,
            live: HashMap::new(),
        }));
        let state_thread = Arc::clone(&state);
        std::thread::spawn(move || thread::run(rx, state_thread));
        Self { tx, state }
    }

    /// Enchufa el bus a esa tarjeta, abriendola si hacia falta. Asincrono, como
    /// lo era el cambio de dispositivo del motor de efectos.
    pub fn set_bus_device(&self, bus: BusId, device_name: &str) {
        let _ = self.tx.send(ConsoleCommand::SetBusDevice {
            bus,
            device_name: device_name.to_string(),
        });
    }

    /// Desconecta el bus. La tarjeta se cierra si no la usa ningun otro.
    pub fn close_bus(&self, bus: BusId) {
        let _ = self.tx.send(ConsoleCommand::CloseBus { bus });
    }

    /// El bus, si existe ahora mismo. Clonar es barato (solo Arcs) y suelta el
    /// candado en el acto: quien reproduce no debe esperar a la consola.
    pub fn bus(&self, bus: BusId) -> Option<Bus> {
        self.state.lock().unwrap().live.get(&bus).cloned()
    }

    /// Los niveles del bus. Existen aunque el bus no tenga tarjeta.
    pub fn levels(&self, bus: BusId) -> (Arc<AtomicU32>, Arc<AtomicU32>) {
        let state = self.state.lock().unwrap();
        let slot = &state.slots[&bus];
        (Arc::clone(&slot.level_l), Arc::clone(&slot.level_r))
    }

    /// El volumen del bus. Existe aunque el bus no tenga tarjeta.
    pub fn volume(&self, bus: BusId) -> Arc<AtomicU32> {
        Arc::clone(&self.state.lock().unwrap().slots[&bus].volume)
    }
}

impl Default for ConsoleEngine {
    fn default() -> Self {
        Self::new()
    }
}
