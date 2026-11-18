use super::{
    bus::{Bus, BusOutput},
    duck::{DuckControl, DuckStage},
    fader::FaderSource,
};
use rodio::{buffer::SamplesBuffer, dynamic_mixer, source::Zero};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
fn gain(v: f32) -> Arc<AtomicU32> {
    Arc::new(AtomicU32::new(v.to_bits()))
}

#[test]
fn independent_destination_applies_live_master_once_and_mic_ceiling() {
    let master = gain(0.5);
    let duck = Arc::new(DuckControl::default());
    let mut source = FaderSource::new(SamplesBuffer::new(2, 48000, vec![1.0; 12]), gain(0.8))
        .with_duck(Some(DuckStage {
            control: duck.clone(),
            parent: Some(master.clone()),
            parent_ramp: None,
        }))
        .with_master(Some(master.clone()));
    assert_eq!(source.next(), Some(0.4));
    assert_eq!(source.next(), Some(0.4));
    master.store(0.25f32.to_bits(), Ordering::Relaxed);
    assert_eq!(source.next(), Some(0.2));
    assert_eq!(source.next(), Some(0.2));
    duck.set(true, 0.1, 0, 0);
    assert!((source.next().unwrap() - 0.1).abs() < 0.00001);
    source.next();
    master.store(0.0f32.to_bits(), Ordering::Relaxed);
    assert_eq!(source.next(), Some(0.0));
    assert_eq!(source.next(), Some(0.0));
    duck.set(false, 0.1, 0, 0);
    master.store(1.5f32.to_bits(), Ordering::Relaxed);
    assert!((source.next().unwrap() - 1.2).abs() < 0.00001);
}

#[test]
fn destination_meter_is_after_master_and_does_not_leak_to_program() {
    let (out, mut destination) = dynamic_mixer::mixer::<f32>(2, 48000);
    out.add(Zero::<f32>::new(2, 48000));
    let (program_out, mut program) = dynamic_mixer::mixer::<f32>(2, 48000);
    program_out.add(Zero::<f32>::new(2, 48000));
    let l = gain(0.0);
    let r = gain(0.0);
    let bus = Bus::open_controlled(
        BusOutput::Bus(&out),
        l.clone(),
        r.clone(),
        gain(0.8),
        None,
        Some(gain(0.5)),
        None,
    )
    .unwrap();
    bus.add(SamplesBuffer::new(2, 48000, vec![1.0; 8192]));
    for _ in 0..4096 {
        assert_eq!(destination.next(), Some(0.4));
        assert_eq!(program.next(), Some(0.0));
    }
    assert!((f32::from_bits(l.load(Ordering::Relaxed)) - 0.4).abs() < 0.00001);
    assert!((f32::from_bits(r.load(Ordering::Relaxed)) - 0.4).abs() < 0.00001);
}
