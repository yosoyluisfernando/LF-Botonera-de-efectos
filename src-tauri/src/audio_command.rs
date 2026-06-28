/// Modulo: audio_command.rs
/// Proposito: mensajes internos que la fachada de audio envia al hilo dedicado.
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
        /// true = a la salida de pre-escucha (si existe); false = principal.
        to_pre: bool,
        /// Tiempo de fade-in al inicio de la reproducción (0.0 = sin fade).
        fade_in_s: f64,
        /// Tiempo de fade-out al pulsar Detener (0.0 = corte inmediato).
        fade_out_stop_s: f64,
        /// Tiempo de fade-out al terminar naturalmente (0.0 = sin fade).
        fade_out_end_s: f64,
    },
    Stop {
        id: String,
    },
    /// Igual que Stop pero con fundido si el ButtonSource fue creado con fade.
    StopFade {
        id: String,
    },
    StopAll,
    /// Igual que StopAll pero con fundido en todos los ButtonSource que lo soporten.
    StopAllFade,
    SetDevice {
        device_name: String,
    },
    /// Fija/limpia el dispositivo de pre-escucha. Vacío = usar el principal.
    SetPreDevice {
        device_name: String,
    },
    SetVolume {
        id: String,
        volume: f32,
    },
    PlaySequence {
        id: String,
        paths: Vec<String>,
        volume: f32,
        duration: f64,
    },
}
