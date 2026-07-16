//! Modulo: player.rs
//! Proposito: configuracion del reproductor auxiliar del panel (modo reproductor).
//! Tipos puros y serializacion; sin logica de audio ni de I/O. La reproduccion
//! vive en `engine/player/`. Ver `Documentacion/PLAN_MODO_REPRODUCTOR.md`.
use super::ButtonData;
use serde::{Deserialize, Serialize};

/// Configuracion persistida del reproductor. Es un unico reproductor global,
/// con su propia cola de pistas, independiente de los botones fijos.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerConfig {
    /// Cola ordenada de pistas. Cada pista reutiliza `ButtonData`, asi soporta
    /// los mismos tipos (audio, carpeta aleatoria, temperatura, humedad, hora).
    #[serde(default)]
    pub tracks: Vec<ButtonData>,
    /// Modo de avance de la cola: "normal" | "repeat" | "random" | "manual".
    #[serde(default = "default_mode")]
    pub playback_mode: String,
    /// Volumen propio del reproductor, independiente del master (0.0..=1.5).
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Dispositivo de salida propio. "" = el mismo dispositivo principal de los
    /// efectos; "default" = predeterminado del sistema; otro = nombre exacto.
    #[serde(default)]
    pub output_device: String,
    /// Que muestra el contador de la pista: "elapsed" | "remaining". Se recuerda
    /// entre sesiones; con el reproductor parado se ensena el total de la lista.
    #[serde(default = "default_time_display")]
    pub time_display: String,
}

/// Modos validos del contador. Fuente unica para validar en el IPC.
pub const TIME_DISPLAYS: [&str; 2] = ["elapsed", "remaining"];

fn default_time_display() -> String {
    "elapsed".into()
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            playback_mode: default_mode(),
            volume: default_volume(),
            output_device: String::new(),
            time_display: default_time_display(),
        }
    }
}

fn default_mode() -> String {
    "normal".into()
}
fn default_volume() -> f32 {
    1.0
}

/// Modos validos de avance de la cola. Fuente unica para validar en el IPC.
pub const PLAYBACK_MODES: [&str; 4] = ["normal", "repeat", "random", "manual"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_product_decisions() {
        let player = PlayerConfig::default();
        assert!(player.tracks.is_empty());
        assert_eq!(player.playback_mode, "normal");
        assert_eq!(player.volume, 1.0);
        assert_eq!(player.output_device, "");
    }

    #[test]
    fn default_mode_is_a_valid_mode() {
        assert!(PLAYBACK_MODES.contains(&default_mode().as_str()));
    }
}
