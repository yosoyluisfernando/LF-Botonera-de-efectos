//! Modulo: engine/player/prefetch.rs
//! Proposito: un colchon de diez segundos entre el disco y la tarjeta.
//!
//! **El problema.** `DeckSource::next()` corre DENTRO del callback de audio, y
//! le pide su muestra al decodificador, que lee del disco. Si el disco se atasca
//! un instante —otro programa copiando, el antivirus despertando, un disco que
//! se dormia— el callback se queda esperando y la tarjeta se queda sin muestras.
//! Eso es el microcorte. No es culpa del formato ni del codec: es que decodificar
//! y leer del disco no tienen nada que hacer dentro de un callback de audio.
//!
//! **La solucion.** Un hilo va decodificando por delante y deja las muestras en
//! una cola acotada. El callback solo saca de la cola, que es mover memoria y no
//! se puede atascar. Con diez segundos por delante, un atasco mas corto que eso
//! no se oye.
//!
//! **Va DEBAJO de `DeckSource`, y esto no es un detalle de colocacion.**
//! `DeckSource` cuenta las muestras que consume, y de ese contador salen tres
//! cosas: la posicion que ve el usuario, el fin de pista que dispara la
//! siguiente, y el segundo por el que se rehace la fuente al cambiar de tarjeta.
//! Con el colchon por ENCIMA, ese contador iria diez segundos por delante de lo
//! que suena: la barra de progreso adelantada, la cancion siguiente entrando
//! diez segundos antes de tiempo y un salto al cambiar de tarjeta. Por debajo,
//! el contador sigue midiendo lo que de verdad sale por el altavoz y no hay que
//! tocar nada mas.
use crate::engine::audio::decode::BoxSource;
use crate::engine::cache::preload::{build_play_source, PreloadCache};
use rodio::Source;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::vec::IntoIter;

/// Segundos por delante. Diez cubre de sobra el atasco de un disco ocupado sin
/// que la memoria se note: a 48 kHz estereo son unos 3,8 MB por deck, y con los
/// dos del ping-pong no llega a 8 MB.
const AHEAD_S: usize = 10;

/// Muestras por lote. Mandarlas de una en una por el canal costaria mas que
/// decodificarlas.
const CHUNK: usize = 4096;

/// La fuente de un deck, ya con su colchon.
///
/// **Todo lo que suene en un deck tiene que salir de aqui**: cargar una pista,
/// saltar de posicion y rehacerla al cambiar de tarjeta. Una via que se saltara
/// el colchon volveria a leer del disco dentro del callback, y los microcortes
/// reapareceria solo al saltar o solo al cambiar de tarjeta — de los bugs mas
/// dificiles de encontrar que hay.
pub fn deck_source(
    cache: &Arc<Mutex<PreloadCache>>,
    path: &str,
    cue_start_s: f64,
    cue_end_s: Option<f64>,
) -> Option<BoxSource> {
    build_play_source(cache, path, false, cue_start_s, cue_end_s).map(buffered)
}

/// Igual, para lo que no sale de la cache: una locucion son varios archivos
/// encadenados.
///
/// Se envuelve tambien lo que ya viene de RAM. Distinguirlo ahorraria un hilo
/// que no hace falta, pero obligaria a que la cache contara por donde salio cada
/// fuente, y una costura asi se paga cara para lo que ahorra: el hilo de una
/// fuente en RAM entrega sus muestras y se muere solo.
pub fn buffered(inner: BoxSource) -> BoxSource {
    Box::new(PrefetchSource::new(inner))
}

struct PrefetchSource {
    rx: Receiver<Vec<f32>>,
    lote: IntoIter<f32>,
    channels: u16,
    sample_rate: u32,
}

impl PrefetchSource {
    fn new(inner: BoxSource) -> Self {
        // Se preguntan ANTES de mandar la fuente al hilo: despues ya no es
        // nuestra, y el bus necesita saber a que ritmo va desde el primer
        // momento.
        let channels = inner.channels().max(1);
        let sample_rate = inner.sample_rate();
        let (tx, rx) = sync_channel(lotes_por_delante(sample_rate, channels));
        std::thread::spawn(move || llenar(inner, tx));
        Self {
            rx,
            lote: Vec::new().into_iter(),
            channels,
            sample_rate,
        }
    }
}

impl Iterator for PrefetchSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if let Some(muestra) = self.lote.next() {
            return Some(muestra);
        }
        // Se acabo el lote y toca el siguiente. `recv` solo bloquea si el hilo no
        // ha llegado, que es justo el atasco que esto evita — y entonces esperar
        // es lo unico honesto: devolver silencio adelantaria el contador con
        // audio que nadie ha oido, y con el la posicion y el fin de pista.
        //
        // `Err` = el hilo termino y no queda nada: la pista se acabo de verdad.
        // Por eso el fin de pista sigue llegando cuando se vacia el colchon y no
        // cuando el disco llega al final, que seria diez segundos antes.
        self.lote = self.rx.recv().ok()?.into_iter();
        self.lote.next()
    }
}

impl Source for PrefetchSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// El hilo: decodifica por delante hasta llenar la cola, y ahi se queda quieto.
///
/// No hace falta pedirle que pare. Cuando el deck suelta su fuente, el `Receiver`
/// muere y `send` devuelve `Err` — tambien si estaba bloqueado esperando sitio.
/// Un hilo que se apaga solo no se puede olvidar de apagar.
fn llenar(mut inner: BoxSource, tx: SyncSender<Vec<f32>>) {
    loop {
        let mut lote = Vec::with_capacity(CHUNK);
        while lote.len() < CHUNK {
            match inner.next() {
                Some(muestra) => lote.push(muestra),
                None => break,
            }
        }
        let ultimo = lote.len() < CHUNK;
        if !lote.is_empty() && tx.send(lote).is_err() {
            return; // Ya no escucha nadie.
        }
        if ultimo {
            return;
        }
    }
}

/// Cuantos lotes caben en `AHEAD_S` segundos. Al menos uno: una fuente rara con
/// un ritmo absurdo no puede dejar la cola a cero, o `send` se bloquearia para
/// siempre.
fn lotes_por_delante(sample_rate: u32, channels: u16) -> usize {
    let por_segundo = sample_rate as usize * channels.max(1) as usize;
    (por_segundo * AHEAD_S / CHUNK).max(1)
}

#[cfg(test)]
#[path = "prefetch_tests.rs"]
mod prefetch_tests;

#[cfg(test)]
#[path = "prefetch_real_tests.rs"]
mod prefetch_real_tests;
