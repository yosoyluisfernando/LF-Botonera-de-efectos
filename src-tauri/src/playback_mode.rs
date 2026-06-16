/// Modulo: playback_mode.rs
/// Proposito: fuente unica de verdad para modos de reproduccion globales.

/// Flags efectivos que el motor de audio necesita para reproducir un archivo.
pub struct PlaybackFlags {
    pub loop_mode: bool,
    pub stop_other: bool,
    pub overlap: bool,
    pub restart: bool,
}

/// Modo global de reproduccion guardado por perfil.
#[derive(Clone, Copy)]
pub enum PlaybackMode {
    Normal,
    Loop,
    Overlap,
    Restart,
}

impl PlaybackMode {
    /// Convierte una cadena persistida o recibida por IPC en un modo valido.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "normal" => Ok(Self::Normal),
            "loop" => Ok(Self::Loop),
            "overlap" => Ok(Self::Overlap),
            "restart" => Ok(Self::Restart),
            "stop_others" => Ok(Self::Normal),
            _ => Err("invalid_playback_mode".to_string()),
        }
    }

    /// Devuelve normal si una configuracion antigua trae un valor desconocido.
    pub fn from_config(value: &str) -> Self {
        Self::parse(value).unwrap_or(Self::Normal)
    }

    /// Valor estable que se guarda en config y consume la interfaz.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Loop => "loop",
            Self::Overlap => "overlap",
            Self::Restart => "restart",
        }
    }

    /// Resuelve los flags finales combinando modo global y flags del boton.
    pub fn resolve_flags(self, button_flags: PlaybackFlags) -> PlaybackFlags {
        match self {
            Self::Normal => button_flags,
            Self::Loop => PlaybackFlags {
                loop_mode: true,
                stop_other: false,
                overlap: false,
                restart: false,
            },
            Self::Overlap => PlaybackFlags {
                loop_mode: false,
                stop_other: false,
                overlap: true,
                restart: false,
            },
            Self::Restart => PlaybackFlags {
                loop_mode: false,
                stop_other: false,
                overlap: false,
                restart: true,
            },
        }
    }
}
