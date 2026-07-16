//! Modulo: domain/player/advance.rs
//! Proposito: regla pura de avance de la cola del reproductor auxiliar. Sin
//! audio ni I/O: dado el modo, el tamano de la cola, la pista actual y la pista
//! marcada como siguiente, decide el indice de la proxima. Es la fuente de
//! verdad de los tres modos. La ejecucion (decks, pre-carga) vive en el motor.
//!
//! El modo dice QUE pista viene; **no** dice si el reproductor se para. De eso se
//! encarga "detener al finalizar", que es un interruptor aparte y se combina con
//! cualquier modo. Hubo un cuarto modo, `manual`, que hacia justo eso: no avanzar
//! solo. Se quito porque duplicaba el interruptor y ademas limitaba, ya que para
//! elegir la siguiente forzaba el orden normal: "manual + aleatorio" era
//! imposible. Con el interruptor, cualquier combinacion funciona.

/// Modo de avance de la cola. Fuente unica de verdad para el reproductor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerMode {
    Normal,
    Repeat,
    Random,
}

impl PlayerMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "normal" => Ok(Self::Normal),
            "repeat" => Ok(Self::Repeat),
            "random" => Ok(Self::Random),
            _ => Err("invalid_player_mode".into()),
        }
    }
    /// Tolerante: una configuracion antigua puede traer el modo `manual`, que ya
    /// no existe. Cae a Normal, que es lo que hacia para elegir la siguiente.
    pub fn from_config(value: &str) -> Self {
        Self::parse(value).unwrap_or(Self::Normal)
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Repeat => "repeat",
            Self::Random => "random",
        }
    }
}

/// Indice de la siguiente pista al terminar `current`.
///
/// - **Lo marcado como siguiente es LEY:** si `marked` es un indice valido, se
///   usa siempre, sin importar el modo (el motor luego lo consume).
/// - Sin marcado: **Normal** avanza y se detiene al final (`None`); **Repeat** da
///   la vuelta; **Random** elige al azar evitando repetir la actual si hay mas de
///   una.
///
/// Siempre responde QUE pista viene. Si el reproductor debe pararse antes de
/// arrancarla es cosa de "detener al finalizar", no del modo.
///
/// `rand` es un valor `0.0..1.0` inyectado para que Random sea probable de forma
/// pura y determinista en las pruebas.
pub fn next_index(
    mode: PlayerMode,
    len: usize,
    current: Option<usize>,
    marked: Option<usize>,
    rand: f64,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    if let Some(m) = marked {
        if m < len {
            return Some(m);
        }
    }
    match mode {
        PlayerMode::Normal => match current {
            Some(c) if c + 1 < len => Some(c + 1),
            None => Some(0),
            _ => None,
        },
        PlayerMode::Repeat => Some(current.map_or(0, |c| (c + 1) % len)),
        PlayerMode::Random => Some(random_index(len, current, rand)),
    }
}

/// Elige un indice al azar evitando repetir `current` cuando hay mas de una.
fn random_index(len: usize, current: Option<usize>, rand: f64) -> usize {
    let r = rand.clamp(0.0, 0.999_999);
    if len == 1 {
        return 0;
    }
    match current {
        Some(c) => {
            let pick = (r * (len - 1) as f64) as usize;
            if pick >= c {
                pick + 1
            } else {
                pick
            }
        }
        None => (r * len as f64) as usize,
    }
}
