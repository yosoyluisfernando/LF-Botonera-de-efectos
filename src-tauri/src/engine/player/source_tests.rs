//! Pruebas de la fuente de un deck. Cubren las tres cosas que antes daba el
//! `Sink` de rodio y ahora hay que sostener a mano: contar la posicion, avisar de
//! que termino, y pausar sin perder el sitio.
use super::{DeckHandle, DeckSource};
use crate::engine::audio::decode::BoxSource;
use rodio::buffer::SamplesBuffer;

/// 4 muestras mono a 4 Hz = 1 segundo de audio. Numeros comodos para que la
/// posicion salga exacta y la prueba diga algo claro.
fn deck(samples: &[f32]) -> (DeckSource, DeckHandle) {
    let buffer = SamplesBuffer::new(1, 4, samples.to_vec());
    DeckSource::new(Box::new(buffer) as BoxSource, 1.0)
}

#[test]
fn arranca_en_pausa_y_no_avanza_hasta_que_se_le_dice() {
    let (mut source, handle) = deck(&[1.0, 1.0, 1.0, 1.0]);
    // En pausa da silencio, no None: seguir en el bus es lo que la distingue de
    // estar parada.
    assert_eq!(source.next(), Some(0.0));
    assert_eq!(source.next(), Some(0.0));
    assert_eq!(handle.position_s(), 0.0);
    assert!(!handle.is_done());
}

#[test]
fn al_reanudar_sigue_por_donde_iba() {
    let (mut source, handle) = deck(&[0.5, 0.25, 0.125, 0.0625]);
    handle.play();
    assert_eq!(source.next(), Some(0.5));
    handle.pause();
    // La pausa no se come muestras: al volver sale la siguiente, no la tercera.
    assert_eq!(source.next(), Some(0.0));
    handle.play();
    assert_eq!(source.next(), Some(0.25));
}

/// Sustituye a `sink.get_pos()`.
#[test]
fn la_posicion_cuenta_lo_reproducido() {
    let (mut source, handle) = deck(&[1.0, 1.0, 1.0, 1.0]);
    handle.play();
    source.next();
    source.next();
    // 2 muestras de 4 por segundo = medio segundo.
    assert_eq!(handle.position_s(), 0.5);
}

/// Sustituye a `sink.empty()`.
#[test]
fn avisa_cuando_se_agota_sola() {
    let (mut source, handle) = deck(&[1.0, 1.0]);
    handle.play();
    assert!(!handle.is_done());
    source.next();
    source.next();
    assert_eq!(source.next(), None);
    assert!(handle.is_done());
}

#[test]
fn parar_la_retira_del_bus_y_avisa() {
    let (mut source, handle) = deck(&[1.0, 1.0, 1.0, 1.0]);
    handle.play();
    source.next();
    handle.stop();
    // None = el mixer la deja caer. Es la unica forma de sacarla de ahi.
    assert_eq!(source.next(), None);
    assert!(handle.is_done());
}

/// La ganancia de la pista SI va en la fuente: es del archivo. El volumen del
/// reproductor no, porque es el fader del bus.
#[test]
fn aplica_la_ganancia_de_la_pista() {
    let buffer = SamplesBuffer::new(1, 4, vec![1.0f32, 1.0]);
    let (mut source, handle) = DeckSource::new(Box::new(buffer) as BoxSource, 0.5);
    handle.play();
    assert_eq!(source.next(), Some(0.5));
}
