//! Modulo: engine/player/deck_track.rs
//! Proposito: los DATOS de un deck — en que estado esta y que pista lleva. El
//! comportamiento vive en `deck.rs`.

/// Estado de un deck. `Loaded` = pre-cargado en pausa, listo para arrancar al
/// instante. `Finished` = termino de forma natural y espera relevo. `Failed` = no
/// se pudo cargar (carpeta vacia, sin clima, archivo ilegible); se trata como
/// "termino" para que el motor releve y la musica siga.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeckStatus {
    #[default]
    Empty,
    Loaded,
    Playing,
    Paused,
    Finished,
    Failed,
}

/// Lo que el deck recuerda de la pista cargada. Hace falta para poder
/// RECONSTRUIRLA en otra posicion: una fuente no se reposiciona, asi que tanto un
/// salto como un cambio de tarjeta son volver a crearla desde el punto pedido.
#[derive(Clone, Default)]
pub struct DeckTrack {
    /// Ruta YA resuelta (la que suena). Para una carpeta aleatoria es la cancion
    /// elegida, no la carpeta: al recargar debe sonar la misma, no otra.
    pub path: String,
    pub duration_s: f64,
    pub gain: f32,
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    /// Una locucion son varios archivos encadenados en una sola fuente: no se
    /// puede reposicionar. La barra de progreso lo respeta y no deja arrastrar, y
    /// un cambio de tarjeta la deja caer en vez de rehacerla mal.
    pub seekable: bool,
}
