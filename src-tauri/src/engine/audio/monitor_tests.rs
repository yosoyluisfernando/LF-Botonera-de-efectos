//! Pruebas del monitor de audio. Van aparte por el limite de 200 lineas (regla 3).
use super::compute_display_time;
use crate::engine::audio::button::{ButtonState, ButtonStateMap, PlaybackGroup};
use crate::engine::audio::last_pressed::LastPressedInfo;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[test]
fn display_time_uses_latest_instance_of_last_pressed_button() {
    let mut map = ButtonStateMap::new();
    map.insert(
        "btn".to_string(),
        vec![
            state_started_ago(10.0, Duration::from_secs(9)),
            state_started_ago(10.0, Duration::from_secs(2)),
        ],
    );
    let last = Mutex::new(Some(LastPressedInfo {
        id: "btn".to_string(),
    }));

    let (remaining, _) = compute_display_time(&map, &last);

    assert!(remaining > 7.5);
}

fn state_started_ago(duration: f64, elapsed: Duration) -> ButtonState {
    ButtonState {
        group: PlaybackGroup::Main,
        done_flag: Arc::new(AtomicBool::new(false)),
        stop_flag: Arc::new(AtomicBool::new(false)),
        fade_out_flag: None,
        volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
        start_time: Instant::now() - elapsed,
        position_offset_s: 0.0,
        duration,
        loop_mode: false,
    }
}
