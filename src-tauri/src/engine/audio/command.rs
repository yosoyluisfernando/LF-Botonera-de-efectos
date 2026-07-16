/// Modulo: audio_command.rs
/// Proposito: mensajes internos que la fachada de audio envia al hilo dedicado.
use crate::engine::audio::button::PlaybackGroup;
use crate::engine::console::{BusId, Routing};

pub enum AudioCommand {
    Play {
        id: String,
        path: String,
        volume: f32,
        duration: f64,
        loop_mode: bool,
        stop_other: bool,
        overlap: bool,
        restart: bool,
        /// Punto de inicio (cue) en segundos.
        cue_start_s: f64,
        /// Punto de fin (recorte); None = hasta el final.
        cue_end_s: Option<f64>,
        /// Ganancia del archivo (capa 1): normalización/dB del editor, lineal.
        file_gain: f32,
        /// El bus de la consola por el que suena. Lo decide `routing::bus_for`.
        bus: BusId,
        /// Tiempo de fade-in al inicio de la reproducción (0.0 = sin fade).
        fade_in_s: f64,
        /// Tiempo de fade-out al pulsar Detener (0.0 = corte inmediato).
        fade_out_stop_s: f64,
        /// Tiempo de fade-out al terminar naturalmente (0.0 = sin fade).
        fade_out_end_s: f64,
        group: PlaybackGroup,
    },
    Stop {
        id: String,
    },
    /// Igual que Stop pero con fundido si el ButtonSource fue creado con fade.
    StopFade {
        id: String,
    },
    StopAll,
    StopGroupFade {
        group: PlaybackGroup,
    },
    /// Igual que StopAll pero con fundido en todos los ButtonSource que lo soporten.
    StopAllFade,
    /// Cambia el ruteo de un bus de la consola. Pasa por el hilo de audio, y no
    /// directo a la consola, para respetar el orden con los Play que ya viajan
    /// por este canal.
    SetBusRouting {
        bus: BusId,
        routing: Routing,
    },
    SetVolume {
        id: String,
        volume: f32,
    },
    SeekActive {
        delta_s: Option<f64>,
        position_s: Option<f64>,
    },
    PlaySequence {
        id: String,
        paths: Vec<String>,
        volume: f32,
        duration: f64,
        bus: BusId,
        group: PlaybackGroup,
    },
}
