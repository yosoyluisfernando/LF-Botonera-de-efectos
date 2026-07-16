//! Pruebas de la cadena de un bus, montada de verdad pero sin tarjeta de sonido:
//! un bus se enchufa a un mixer que hace de programa, y se consume a mano para
//! ver que sale y que mide el medidor.
//!
//! Es lo mas cerca del audio real que se puede llegar sin hardware, y cubre lo
//! que de verdad importa: que el fader escale la suma, que el medidor mida
//! DESPUES del fader, y que dos buses en el mismo padre se sumen sin tocarse.
use super::{Bus, BusOutput};
use rodio::buffer::SamplesBuffer;
use rodio::dynamic_mixer::{self, DynamicMixerController};
use rodio::source::{Source, Zero};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

const CH: u16 = 2;
const RATE: u32 = 48_000;
/// El medidor promedia por ventanas de 1024 muestras: hay que consumir al menos
/// una entera para que publique algo.
const WINDOW: usize = 1024;

struct Rig {
    controller: Arc<DynamicMixerController<f32>>,
    mixer: Box<dyn Source<Item = f32> + Send>,
}

/// Un mixer que hace de bus de programa: es lo que la consola monta de verdad,
/// solo que aqui lo consumimos nosotros en vez de una tarjeta.
fn program() -> Rig {
    let (controller, mixer) = dynamic_mixer::mixer::<f32>(CH, RATE);
    controller.add(Zero::<f32>::new(CH, RATE));
    Rig {
        controller,
        mixer: Box::new(mixer),
    }
}

fn tone(level: f32, samples: usize) -> SamplesBuffer<f32> {
    SamplesBuffer::new(CH, RATE, vec![level; samples])
}

fn atomics(gain: f32) -> (Arc<AtomicU32>, Arc<AtomicU32>, Arc<AtomicU32>) {
    (
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(gain.to_bits())),
    )
}

fn read(level: &Arc<AtomicU32>) -> f32 {
    f32::from_bits(level.load(Ordering::Relaxed))
}

/// Consume una ventana entera del programa y devuelve el pico de lo que salio.
fn drain(rig: &mut Rig, windows: usize) -> f32 {
    let mut peak: f32 = 0.0;
    for _ in 0..(WINDOW * windows) {
        if let Some(s) = rig.mixer.next() {
            peak = peak.max(s.abs());
        }
    }
    peak
}

#[test]
fn el_bus_entrega_su_senal_al_programa() {
    let mut rig = program();
    let (l, r, gain) = atomics(1.0);
    let bus = Bus::open(BusOutput::Bus(&rig.controller), l, r, gain).unwrap();
    bus.add(tone(0.5, WINDOW * 4));
    assert!((drain(&mut rig, 1) - 0.5).abs() < 1e-6);
}

/// El fader del bus escala su suma antes de entregarla.
#[test]
fn el_fader_del_bus_escala_lo_que_entrega() {
    let mut rig = program();
    let (l, r, gain) = atomics(0.5);
    let bus = Bus::open(BusOutput::Bus(&rig.controller), Arc::clone(&l), r, gain).unwrap();
    bus.add(tone(1.0, WINDOW * 4));
    assert!((drain(&mut rig, 1) - 0.5).abs() < 1e-6);
    // Y el medidor cuenta lo que SALE, no lo que entro: va despues del fader.
    assert!((read(&l) - 0.5).abs() < 1e-6);
}

/// La regla que hace honesto al vumetro: el medidor va DESPUES del fader, asi que
/// la aguja siempre enseña lo que de verdad sale. Al reves no se enteraria de los
/// movimientos del fader.
#[test]
fn mover_el_fader_mueve_el_medidor() {
    let mut rig = program();
    let (l, r, gain) = atomics(1.0);
    let bus = Bus::open(
        BusOutput::Bus(&rig.controller),
        Arc::clone(&l),
        r,
        Arc::clone(&gain),
    )
    .unwrap();
    bus.add(tone(1.0, WINDOW * 8));
    drain(&mut rig, 1);
    let alto = read(&l);
    gain.store(0.25f32.to_bits(), Ordering::Relaxed);
    drain(&mut rig, 1);
    let bajo = read(&l);
    assert!(alto > 0.9, "con el fader arriba el medidor debe marcar: {alto}");
    assert!(
        (bajo - 0.25).abs() < 1e-6,
        "al bajar el fader el medidor debe seguirlo: {bajo}"
    );
}

/// El fader a cero: silencio de verdad y aguja a cero, sin esperar a nada.
#[test]
fn el_fader_a_cero_deja_el_medidor_a_cero() {
    let mut rig = program();
    let (l, _r, gain) = atomics(0.0);
    let bus = Bus::open(BusOutput::Bus(&rig.controller), Arc::clone(&l), _r, gain).unwrap();
    bus.add(tone(1.0, WINDOW * 4));
    assert_eq!(drain(&mut rig, 1), 0.0);
    assert_eq!(read(&l), 0.0);
}

/// Dos buses en el mismo padre se suman ahi, y **el fader de uno no toca al
/// otro**. Es lo que separa "bajar la musica para hablar" de "bajarlo todo".
#[test]
fn el_fader_de_un_bus_no_toca_al_otro() {
    let mut rig = program();
    let (l1, r1, gain1) = atomics(1.0);
    let (l2, r2, gain2) = atomics(1.0);
    let uno = Bus::open(BusOutput::Bus(&rig.controller), Arc::clone(&l1), r1, gain1).unwrap();
    let dos = Bus::open(BusOutput::Bus(&rig.controller), Arc::clone(&l2), r2, Arc::clone(&gain2)).unwrap();
    uno.add(tone(0.5, WINDOW * 8));
    dos.add(tone(0.5, WINDOW * 8));
    drain(&mut rig, 1);
    assert!((read(&l1) - 0.5).abs() < 1e-6);
    // Bajar el segundo a cero deja el primero intacto.
    gain2.store(0.0f32.to_bits(), Ordering::Relaxed);
    drain(&mut rig, 1);
    assert!(
        (read(&l1) - 0.5).abs() < 1e-6,
        "bajar un bus no debe tocar al otro: {}",
        read(&l1)
    );
    assert_eq!(read(&l2), 0.0);
}

/// Un bus sin fuentes no se muere ni mata al padre: el `Zero` lo mantiene vivo
/// dando silencio, y por eso las fuentes que lleguen despues sí suenan.
#[test]
fn un_bus_vacio_da_silencio_pero_sigue_vivo() {
    let mut rig = program();
    let (l, r, gain) = atomics(1.0);
    let bus = Bus::open(BusOutput::Bus(&rig.controller), Arc::clone(&l), r, gain).unwrap();
    assert_eq!(drain(&mut rig, 1), 0.0);
    // Y ahora que ya estuvo un rato vacio, todavia acepta y suena.
    bus.add(tone(0.75, WINDOW * 4));
    assert!((drain(&mut rig, 1) - 0.75).abs() < 1e-6);
}
