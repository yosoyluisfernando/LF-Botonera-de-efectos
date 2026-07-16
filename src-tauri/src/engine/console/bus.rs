/// Modulo: engine/console/bus.rs
/// Proposito: Bus = un punto de suma con su medidor. Mixer + LevelSource,
/// enchufado a un endpoint con `play_raw`.
///
/// Es el MasterBus de antes MENOS el Sink, y esa amputacion es el corazon de la
/// consola: separa "soy una senal" de "soy un conector". El Sink no se sustituye
/// por otra cosa, se cambia por `play_raw`: un bus nunca se pausa, asi que esa
/// capa de control sobraba.
///
/// El bus tampoco sabe ya que es un boton: solo acepta fuentes. Quien construye
/// un ButtonSource con sus fades y su estado es el motor de efectos
/// (`engine/audio/attach.rs`), que es quien sabe de botones.
///
/// La cadena del bus, en orden:
///
/// ```text
///   fuentes → DynamicMixer → FaderSource → LevelSource → play_raw(endpoint)
/// ```
///
/// **El medidor va DESPUES del fader**, y no es un detalle: asi el vumetro
/// enseña lo que de verdad sale (baja el fader, baja la aguja), que es como se
/// comportaba cuando cada fuente aplicaba el master antes de entrar al mixer, y
/// es lo que hace una consola con el medidor de programa. Puesto al reves
/// mediria la señal antes del fader y la aguja no se enteraria de nada.
use super::fader::FaderSource;
use super::level::LevelSource;
use rodio::dynamic_mixer::{self, DynamicMixerController};
use rodio::source::Zero;
use rodio::{OutputStreamHandle, Source};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

const BUS_CHANNELS: u16 = 2;
const BUS_SAMPLE_RATE: u32 = 48_000;

/// Handle de un bus. Todo son Arc, asi que se clona barato y viaja a los demas
/// motores: `DynamicMixerController::add` toma `&self` y el Arc es Send + Sync,
/// de modo que cada motor anade fuentes desde SU hilo. Por eso reproducir no
/// necesita pasar por el hilo de la consola.
#[derive(Clone)]
pub struct Bus {
    controller: Arc<DynamicMixerController<f32>>,
    level_l: Arc<AtomicU32>,
    level_r: Arc<AtomicU32>,
}

/// Donde entrega un bus lo que sale de su cadena.
pub enum BusOutput<'a> {
    /// A una tarjeta. Es el final del camino.
    Endpoint(&'a OutputStreamHandle),
    /// A otro bus, que lo suma con los demas. Asi entran los buses en el
    /// programa: el mixer de un bus es una fuente mas del mixer del PGM.
    Bus(&'a Arc<DynamicMixerController<f32>>),
}

impl Bus {
    /// Crea el bus y lo enchufa a su destino. None si `play_raw` falla (la
    /// tarjeta se perdio entre medias). `gain` es el fader: se mueve mientras
    /// suena.
    pub fn open(
        out: BusOutput,
        level_l: Arc<AtomicU32>,
        level_r: Arc<AtomicU32>,
        gain: Arc<AtomicU32>,
    ) -> Option<Self> {
        let (controller, mixer) = dynamic_mixer::mixer::<f32>(BUS_CHANNELS, BUS_SAMPLE_RATE);
        // SIN esto el DynamicMixer devuelve None en cuanto se queda vacio, la
        // salida lo da por terminado y las fuentes anadidas despues no suenan.
        controller.add(Zero::<f32>::new(BUS_CHANNELS, BUS_SAMPLE_RATE));
        let faded = FaderSource::new(mixer, gain);
        let measured = LevelSource::new(faded, Arc::clone(&level_l), Arc::clone(&level_r));
        match out {
            BusOutput::Endpoint(handle) => handle.play_raw(measured).ok()?,
            BusOutput::Bus(parent) => parent.add(measured),
        }
        Some(Self {
            controller,
            level_l,
            level_r,
        })
    }

    /// El controller, para que otro bus pueda enchufarse a este.
    pub fn controller(&self) -> &Arc<DynamicMixerController<f32>> {
        &self.controller
    }

    /// Anade una fuente al bus. La consola no sabe que es: puede ser un boton,
    /// una locucion o una cancion.
    pub fn add<S>(&self, source: S)
    where
        S: Source<Item = f32> + Send + 'static,
    {
        self.controller.add(source);
    }

    /// Nivel medido del bus: (L, R). Pico de la ventana mas reciente.
    pub fn levels(&self) -> (Arc<AtomicU32>, Arc<AtomicU32>) {
        (Arc::clone(&self.level_l), Arc::clone(&self.level_r))
    }
}

#[cfg(test)]
#[path = "bus_tests.rs"]
mod bus_tests;
