use super::*;
use crate::engine::console::fader::FaderSource;
use rodio::buffer::SamplesBuffer;

#[test]
fn ramps_are_stereo_coherent_and_preserve_base_faders() {
    let gain = Arc::new(AtomicU32::new(0.6f32.to_bits()));
    let control = Arc::new(DuckControl::default());
    control.set(true, 0.2, 100, 100);
    let samples = SamplesBuffer::new(2, 1000, vec![1.0; 800]);
    let mut source = FaderSource::new(samples, Arc::clone(&gain)).with_duck(Some(DuckStage {
        control: Arc::clone(&control),
        parent: None,
    }));
    let mut last = 0.6;
    for _ in 0..100 {
        let l = source.next().unwrap();
        assert_eq!(Some(l), source.next());
        assert!(l <= last + 0.00001);
        last = l;
    }
    assert!((last - 0.2).abs() < 0.00001);
    assert_eq!(f32::from_bits(gain.load(Ordering::Relaxed)), 0.6);
    control.set(false, 0.2, 100, 100);
    for _ in 0..100 {
        last = source.next().unwrap();
        source.next();
    }
    assert!((last - 0.6).abs() < 0.00001);
}

#[test]
fn reverse_and_rebuild_continue_existing_envelope() {
    let control = Arc::new(DuckControl::default());
    control.set(true, 0.2, 100, 100);
    let stage = DuckStage {
        control: Arc::clone(&control),
        parent: None,
    };
    for _ in 0..50 {
        stage.gain(1.0, 1000);
    }
    let before = stage.gain(1.0, 1000);
    control.set(false, 0.2, 100, 100);
    let rebuilt = DuckStage {
        control,
        parent: None,
    };
    let after = rebuilt.gain(1.0, 1000);
    assert!(after > before && after - before < 0.01);
}

#[test]
fn absolute_level_accounts_for_master_and_new_user_volume() {
    let master = Arc::new(AtomicU32::new(0.6f32.to_bits()));
    let control = Arc::new(DuckControl::default());
    control.set(true, 0.2, 0, 0);
    let stage = DuckStage {
        control: Arc::clone(&control),
        parent: Some(Arc::clone(&master)),
    };
    assert!((stage.gain(1.0, 48000) * 0.6 - 0.2).abs() < 0.00001);
    master.store(0.4f32.to_bits(), Ordering::Relaxed);
    assert!((stage.gain(1.0, 48000) * 0.4 - 0.2).abs() < 0.00001);
    control.set(false, 0.2, 0, 0);
    assert_eq!(stage.gain(1.0, 48000) * 0.4, 0.4);
}

#[test]
fn all_target_combinations_preserve_unselected_groups() {
    for mask in 1..8 {
        for group in 0..3 {
            let selected = mask & (1 << group) != 0;
            let control = Arc::new(DuckControl::default());
            control.set(selected, 0.2, 0, 0);
            let stage = DuckStage {
                control,
                parent: None,
            };
            assert!((stage.gain(0.6, 48000) - if selected { 0.2 } else { 0.6 }).abs() < 0.00001);
        }
    }
    // Cue and Programa retain an ordinary fader without a Mic stage.
    let source = SamplesBuffer::new(2, 48000, vec![1.0, 1.0]);
    let gain = Arc::new(AtomicU32::new(0.6f32.to_bits()));
    assert_eq!(
        FaderSource::new(source, gain).collect::<Vec<_>>(),
        vec![0.6, 0.6]
    );
}
