/// Modulo: engine/audio/tick.rs
/// Proposito: lo que viaja en cada "audio-tick" y como se lee de la consola.
/// Separado de `monitor.rs`, que es el hilo que lo emite.
use crate::engine::console::{BusId, ConsoleEngine};
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Serialize, Clone)]
pub struct TickInfo {
    pub id: String,
    pub pos: f64,
    pub remaining: f64,
    pub duration: f64,
    pub group: &'static str,
    pub progress_percent: f64,
}

/// El pico L/R de un bus en la ventana mas reciente.
#[derive(Serialize, Clone, Copy, Default)]
pub struct BusLevel {
    /// Lineal 0..1, que es lo que necesita la altura de la barra.
    pub l: f32,
    pub r: f32,
    /// El pico de los dos canales en dB, para el numerito de la tira.
    ///
    /// `None` es el silencio: en decibelios el cero no existe —es menos
    /// infinito—, y un numero centinela acabaria pintado en pantalla algun dia.
    /// Lo calcula Rust porque es una escala, no un formato (regla 4).
    pub db: Option<f32>,
}

impl BusLevel {
    fn new(l: f32, r: f32) -> Self {
        let peak = l.max(r);
        let db = (peak > 0.0).then(|| 20.0 * peak.log10());
        Self { l, r, db }
    }
}

/// El nivel de cada bus de la consola. Lo que la UI necesita para pintar una tira
/// de canal por bus.
///
/// El programa no esta aqui: va en `master_level_l/r`, donde el vumetro de la
/// barra inferior lo lee desde siempre.
#[derive(Serialize, Clone, Copy, Default)]
pub struct BusLevels {
    pub efectos: BusLevel,
    pub panel: BusLevel,
    pub reproductor: BusLevel,
    pub cue: BusLevel,
}

#[derive(Serialize, Clone)]
pub struct AudioTickPayload {
    pub buttons: Vec<TickInfo>,
    /// Tiempo restante en segundos para el reloj de la barra inferior.
    pub display_remaining: f64,
    /// Duración original del audio que gobierna el contador de la barra inferior.
    pub display_duration: f64,
    /// El nivel del bus `Programa`: lo que sale al aire, ya con el master.
    pub master_level_l: f32,
    pub master_level_r: f32,
    /// El nivel de cada bus por separado, para la consola.
    pub buses: BusLevels,
    /// Ya no suena nada en el programa: ni efectos, ni panel, ni reproductor.
    /// Este es el ÚLTIMO tick antes del silencio, y va con nivel cero.
    ///
    /// Lo decide Rust porque solo Rust lo sabe (regla 4): el frontend lo deducía
    /// de que la lista de botones viniera vacía, y con música de fondo sin
    /// efectos eso es falso — hay señal de sobra. El vúmetro daba entonces cada
    /// tick por final y le ponía el decaimiento largo, así que la aguja nunca
    /// alcanzaba el nivel real.
    pub idle: bool,
}

/// Los atomicos de nivel de todos los buses, pedidos UNA vez.
///
/// Se guardan y no se vuelven a pedir: son del `BusSlot` y sobreviven a que el
/// grafo se rehaga, asi que valen para toda la vida del monitor. Pedirlos en cada
/// tick seria tomar el candado de la consola diez veces por segundo para nada.
pub struct LevelTaps {
    programa: (Arc<AtomicU32>, Arc<AtomicU32>),
    efectos: (Arc<AtomicU32>, Arc<AtomicU32>),
    panel: (Arc<AtomicU32>, Arc<AtomicU32>),
    reproductor: (Arc<AtomicU32>, Arc<AtomicU32>),
    cue: (Arc<AtomicU32>, Arc<AtomicU32>),
}

impl LevelTaps {
    pub fn new(console: &ConsoleEngine) -> Self {
        Self {
            programa: console.levels(BusId::Programa),
            efectos: console.levels(BusId::Efectos),
            panel: console.levels(BusId::Panel),
            reproductor: console.levels(BusId::Reproductor),
            cue: console.levels(BusId::Cue),
        }
    }

    /// El nivel del programa: (L, R).
    pub fn program(&self) -> (f32, f32) {
        read(&self.programa)
    }

    /// El nivel de cada bus. En reposo van todos a cero, por lo mismo que el
    /// programa: los atomicos aun pueden retener el ultimo pico medido.
    ///
    /// Tambien el CUE, y no es un descuido: la pre-escucha suena por el motor de
    /// efectos y cuenta como boton, asi que si estuviera sonando no habria reposo.
    pub fn buses(&self, idle: bool) -> BusLevels {
        if idle {
            return BusLevels::default();
        }
        let de = |tap: &(Arc<AtomicU32>, Arc<AtomicU32>)| {
            let (l, r) = read(tap);
            BusLevel::new(l, r)
        };
        BusLevels {
            efectos: de(&self.efectos),
            panel: de(&self.panel),
            reproductor: de(&self.reproductor),
            cue: de(&self.cue),
        }
    }
}

fn read(tap: &(Arc<AtomicU32>, Arc<AtomicU32>)) -> (f32, f32) {
    (
        f32::from_bits(tap.0.load(Ordering::Relaxed)),
        f32::from_bits(tap.1.load(Ordering::Relaxed)),
    )
}
