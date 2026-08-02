use super::midi::MidiEngine;
use std::time::{Duration, Instant};

#[test]
fn cancellation_wakes_capture_without_waiting_for_timeout() {
    let engine = MidiEngine::new();
    let wait = engine.begin_capture().unwrap();
    engine.cancel_capture();

    let started = Instant::now();
    let error = wait.wait(Duration::from_secs(1)).unwrap_err();

    assert_eq!(error, "midi_capture_cancelled");
    assert!(started.elapsed() < Duration::from_millis(250));
}

#[test]
fn timeout_releases_capture_slot() {
    let engine = MidiEngine::new();
    let first = engine.begin_capture().unwrap();

    assert_eq!(
        first.wait(Duration::from_millis(1)).unwrap_err(),
        "midi_capture_timeout"
    );
    assert!(engine.begin_capture().is_ok());
}
