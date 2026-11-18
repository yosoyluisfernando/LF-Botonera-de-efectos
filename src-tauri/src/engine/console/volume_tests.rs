//! Límites de canal, ganancia unitaria y estado común de los controles.
use super::{
    bus::BusOutput, controls::ControlTaps, Bus, BusId, BusSlot, ConsoleEngine, ConsoleState,
};
use rodio::{buffer::SamplesBuffer, dynamic_mixer, source::Zero};
use std::sync::{mpsc::channel, Arc, Mutex};
fn console() -> ConsoleEngine {
    let (tx, _rx) = channel();
    let slots = BusId::every()
        .map(|id| (id, BusSlot::new(id.default_routing())))
        .collect();
    ConsoleEngine {
        tx,
        state: Arc::new(Mutex::new(ConsoleState {
            slots,
            ..Default::default()
        })),
    }
}
#[test]
fn every_non_master_bus_is_bounded_at_unity_including_saved_values() {
    let console = console();
    for bus in BusId::every().filter(|b| *b != BusId::Programa) {
        console.set_fader(bus, 1.5);
        assert_eq!(console.fader(bus), 1.0);
        console.set_fader(bus, -0.5);
        assert_eq!(console.fader(bus), 0.0);
    }
    let player: crate::model::player::PlayerConfig =
        serde_json::from_str(r#"{"volume":1.5}"#).unwrap();
    assert_eq!(player.volume, 1.0);
    let faders: crate::model::console::ConsoleConfig = serde_json::from_str(
        r#"{"efectos":1.2,"panel":1.5,"cue":1.5,"output_faders":{"USB":1.5}}"#,
    )
    .unwrap();
    assert_eq!(
        (
            faders.efectos,
            faders.panel,
            faders.cue,
            faders.output_faders["USB"]
        ),
        (1.0, 1.0, 1.0, 1.0)
    );
}
#[test]
fn shared_snapshot_reads_live_volume_and_boost_changes_the_master_only() {
    let console = console();
    let taps = ControlTaps::new(&console);
    let mut cfg = crate::model::AppConfig::default();
    console.set_fader(BusId::Reproductor, 0.37);
    console.set_fader(BusId::Programa, 0.42);
    let normal = taps.snapshot(&cfg).unwrap();
    assert_eq!(normal.player_volume, 0.37);
    assert_eq!(normal.master.volume, 0.42);
    assert_eq!(normal.faders["programa"].max, 1.0);
    cfg.active_profile_mut().unwrap().audio.master_volume_boost = true;
    let boosted = taps.snapshot(&cfg).unwrap();
    assert_ne!(normal, boosted);
    assert_eq!(boosted.master.max, 1.5);
    assert_eq!(boosted.faders["programa"].max, 1.5);
    assert!(boosted
        .faders
        .iter()
        .filter(|(name, _)| name.as_str() != "programa")
        .all(|(_, f)| f.max == 1.0));
}
#[test]
fn console_player_fader_is_the_same_single_gain_and_master_is_the_only_boost() {
    let console = console();
    let (out, mut samples) = dynamic_mixer::mixer::<f32>(2, 48000);
    out.add(Zero::<f32>::new(2, 48000));
    let (l, r) = console.levels(BusId::Programa);
    let master = Bus::open(
        BusOutput::Bus(&out),
        l,
        r,
        console.volume_handle(BusId::Programa),
    )
    .unwrap();
    let (l, r) = console.levels(BusId::Reproductor);
    let player = Bus::open(
        BusOutput::Bus(master.controller()),
        l,
        r,
        console.volume_handle(BusId::Reproductor),
    )
    .unwrap();
    player.add(SamplesBuffer::new(2, 48000, vec![1.0; 128]));
    assert_eq!(samples.next(), Some(1.0));
    assert_eq!(samples.next(), Some(1.0));
    console.set_fader(BusId::Reproductor, 1.5);
    assert_eq!(samples.next(), Some(1.0));
    assert_eq!(samples.next(), Some(1.0));
    console.set_fader(BusId::Programa, 1.5);
    assert_eq!(samples.next(), Some(1.5));
    assert_eq!(samples.next(), Some(1.5));
    console.set_fader(BusId::Reproductor, 0.5);
    assert_eq!(samples.next(), Some(0.75));
    assert_eq!(samples.next(), Some(0.75));
}

#[test]
fn disabling_boost_preserves_all_unity_or_lower_values_and_manual_moves_cancel_ramp() {
    let console = console();
    for value in [0.0, 0.01, 0.25, 0.8, 0.9, 1.0] {
        console.set_fader(BusId::Programa, value);
        assert_eq!(console.limit_master_to_unity(), value);
        assert_eq!(console.fader(BusId::Programa), value);
        assert_eq!(console.state.lock().unwrap().master_ramp.gain(value), value);
    }
    console.set_fader(BusId::Programa, 1.5);
    assert_eq!(console.limit_master_to_unity(), 1.0);
    let ramp = Arc::clone(&console.state.lock().unwrap().master_ramp);
    assert!(ramp.gain(1.0) > 1.0);
    console.set_fader(BusId::Programa, 0.25);
    assert_eq!(ramp.gain(console.fader(BusId::Programa)), 0.25);
}
#[test]
fn program_and_independent_sources_share_smooth_master_but_cue_does_not() {
    use super::fader::FaderSource;
    let console = console();
    console.set_fader(BusId::Programa, 1.5);
    let ramp = Arc::clone(&console.state.lock().unwrap().master_ramp);
    let source = || SamplesBuffer::new(2, 48000, vec![1.0; 8]);
    let mut program = FaderSource::new(source(), console.volume_handle(BusId::Programa))
        .with_ramp(Some(Arc::clone(&ramp)));
    let mut independent = FaderSource::new(source(), console.volume_handle(BusId::Salida(0)))
        .with_master(Some(console.volume_handle(BusId::Programa))).with_ramp(Some(ramp));
    let mut cue = FaderSource::new(source(), console.volume_handle(BusId::Cue));
    console.limit_master_to_unity();
    let p = program.next().unwrap();
    let i = independent.next().unwrap();
    assert!(p > 1.0 && p <= 1.5);
    assert!((p - i).abs() < 0.01);
    assert_eq!(cue.next(), Some(1.0));
    assert_eq!(program.next(), Some(p)); // Pareja estéreo con idéntica ganancia.
}

#[test]
fn microphone_ceiling_uses_effective_master_during_boost_reduction() {
    let console = console();
    console.set_fader(BusId::Programa, 1.5);
    let control = Arc::new(super::duck::DuckControl::default());
    control.set(true, 0.1, 0, 0);
    let ramp = Arc::clone(&console.state.lock().unwrap().master_ramp);
    let stage = super::duck::DuckStage {
        control,
        parent: Some(console.volume_handle(BusId::Programa)),
        parent_ramp: Some(Arc::clone(&ramp)),
    };
    console.limit_master_to_unity();
    let attenuated = stage.gain(1.0, 48000);
    assert!((attenuated * ramp.gain(1.0) - 0.1).abs() < 0.0001);
}
