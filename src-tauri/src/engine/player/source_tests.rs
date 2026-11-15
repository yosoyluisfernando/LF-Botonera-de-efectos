//! Pruebas de la fuente de un deck. Cubren las tres cosas que antes daba el
//! `Sink` de rodio y ahora hay que sostener a mano: contar la posicion, avisar de
//! que termino, y pausar sin perder el sitio.
use super::{DeckHandle, DeckSource};
use crate::engine::audio::attach::{attach_button, AttachArgs};
use crate::engine::audio::button::PlaybackGroup;
use crate::engine::audio::prefetch::PlaybackSource;
use crate::engine::console::bus::{Bus, BusOutput};
use rodio::buffer::SamplesBuffer;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

/// 4 muestras mono a 4 Hz = 1 segundo de audio. Numeros comodos para que la
/// posicion salga exacta y la prueba diga algo claro.
fn deck(samples: &[f32]) -> (DeckSource, DeckHandle) {
    let buffer = SamplesBuffer::new(1, 4, samples.to_vec());
    DeckSource::new(PlaybackSource::InMemory(Box::new(buffer)), 1.0)
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
    let (mut source, handle) = DeckSource::new(PlaybackSource::InMemory(Box::new(buffer)), 0.5);
    handle.play();
    assert_eq!(source.next(), Some(0.5));
}

#[test]
fn cuatro_efectos_siguen_sonando_si_el_reproductor_agota_su_pcm() {
    let (input, producer) = PlaybackSource::queued_stereo(vec![0.25, 0.375]);
    let (player, handle) = DeckSource::new(input, 1.0);
    let (bus, mut output) = rodio::dynamic_mixer::mixer(2, 48_000);
    bus.add(player);
    for _ in 0..4 {
        bus.add(SamplesBuffer::new(2, 48_000, vec![0.0625; 16]));
    }
    handle.play();
    assert_eq!(output.next(), Some(0.5));
    assert_eq!(output.next(), Some(0.625));
    let position = handle.position_s();

    // El productor sigue vivo, pero no entrega el siguiente bloque.
    assert_eq!(output.next(), Some(0.25));
    assert_eq!(handle.position_s(), position);
    assert!(!handle.is_done());

    // Aunque llegue el PCM entre L y R, el deck espera al siguiente frame.
    producer.send(vec![0.125, 0.25]).unwrap();
    assert_eq!(output.next(), Some(0.25));
    assert_eq!(output.next(), Some(0.375));
    assert_eq!(output.next(), Some(0.5));
    assert_eq!(handle.position_s(), position * 2.0);
    drop(producer);
    assert_eq!(output.next(), Some(0.25));
    assert!(handle.is_done());
}

#[test]
fn un_boton_frio_del_panel_no_detiene_otros_buses() {
    let (program, mut output) = rodio::dynamic_mixer::mixer(2, 48_000);
    let open_bus = || {
        Bus::open(
            BusOutput::Bus(&program),
            Arc::default(),
            Arc::default(),
            Arc::new(AtomicU32::new(1.0_f32.to_bits())),
        )
        .unwrap()
    };
    let panel = open_bus();
    let effects = open_bus();
    effects.add(SamplesBuffer::new(2, 48_000, vec![0.25; 16]));
    let (input, producer) = PlaybackSource::queued_stereo(vec![0.125, 0.25]);
    let button = attach_button(
        &panel,
        input,
        AttachArgs {
            volume: 1.0,
            duration: 1.0,
            loop_mode: false,
            file_gain: 1.0,
            fade_in_s: 0.0,
            fade_out_stop_s: 0.0,
            fade_out_end_s: 0.0,
            position_offset_s: 0.0,
            group: PlaybackGroup::Fixed,
            replay: None,
        },
    );
    let render_frame =
        |output: &mut rodio::dynamic_mixer::DynamicMixer<f32>| [output.next(), output.next()];
    assert_eq!(render_frame(&mut output), [Some(0.375), Some(0.5)]);
    assert_eq!(render_frame(&mut output), [Some(0.25), Some(0.25)]);
    assert!(!button.is_done());
    producer.send(vec![0.25, 0.125]).unwrap();
    assert_eq!(render_frame(&mut output), [Some(0.5), Some(0.375)]);
    drop(producer);
    assert_eq!(render_frame(&mut output), [Some(0.25), Some(0.25)]);
    assert!(button.is_done());
}
