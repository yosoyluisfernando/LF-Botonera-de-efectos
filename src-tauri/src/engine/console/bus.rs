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
    /// Volumen master compartido con las fuentes. Sigue aqui porque hoy lo aplica
    /// cada ButtonSource por su cuenta; en la Fase 2 pasa a ser el fader del bus
    /// y desaparece de las fuentes.
    master_volume: Arc<AtomicU32>,
}

impl Bus {
    /// Crea el bus y lo enchufa al endpoint. None si `play_raw` falla (la tarjeta
    /// se perdio entre medias).
    pub fn open(
        handle: &OutputStreamHandle,
        level_l: Arc<AtomicU32>,
        level_r: Arc<AtomicU32>,
        master_volume: Arc<AtomicU32>,
    ) -> Option<Self> {
        let (controller, mixer) = dynamic_mixer::mixer::<f32>(BUS_CHANNELS, BUS_SAMPLE_RATE);
        // SIN esto el DynamicMixer devuelve None en cuanto se queda vacio, la
        // salida lo da por terminado y las fuentes anadidas despues no suenan.
        controller.add(Zero::<f32>::new(BUS_CHANNELS, BUS_SAMPLE_RATE));
        let measured = LevelSource::new(mixer, Arc::clone(&level_l), Arc::clone(&level_r));
        handle.play_raw(measured).ok()?;
        Some(Self {
            controller,
            level_l,
            level_r,
            master_volume,
        })
    }

    /// Anade una fuente al bus. La consola no sabe que es: puede ser un boton,
    /// una locucion o una cancion.
    pub fn add<S>(&self, source: S)
    where
        S: Source<Item = f32> + Send + 'static,
    {
        self.controller.add(source);
    }

    /// El atomico del volumen master, para las fuentes que aun se lo aplican solas.
    pub fn master_volume(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.master_volume)
    }

    /// Nivel medido del bus: (L, R). Pico de la ventana mas reciente.
    pub fn levels(&self) -> (Arc<AtomicU32>, Arc<AtomicU32>) {
        (Arc::clone(&self.level_l), Arc::clone(&self.level_r))
    }
}
