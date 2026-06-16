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
    },
    Stop {
        id: String,
    },
    StopAll,
    SetDevice {
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
