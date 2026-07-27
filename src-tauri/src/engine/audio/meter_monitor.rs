//! Emite únicamente niveles de audio a 50 FPS para los vúmetros.
use super::tick::LevelTaps;
use crate::engine::console::ConsoleEngine;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

const METER_TICK: Duration = Duration::from_millis(20);

pub fn start(app: tauri::AppHandle, console: Arc<ConsoleEngine>, idle: Arc<AtomicBool>) {
    thread::spawn(move || {
        let taps = LevelTaps::new(&console);
        let mut was_idle = false;
        loop {
            let is_idle = idle.load(Ordering::Acquire);
            if !is_idle || !was_idle {
                let _ = app.emit("meter-tick", taps.snapshot(is_idle));
            }
            was_idle = is_idle;
            thread::sleep(METER_TICK);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::METER_TICK;

    #[test]
    fn meter_contract_is_fifty_updates_per_second() {
        assert_eq!(METER_TICK.as_millis(), 1000 / 50);
    }
}
