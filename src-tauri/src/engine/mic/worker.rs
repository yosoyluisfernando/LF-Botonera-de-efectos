use super::{
    capture::{self, Meter},
    Command, Snapshot,
};
use crate::{
    domain::mic::MicState,
    engine::console::{BusId, ConsoleEngine},
    model::{mic::MicMode, AppConfig},
};
use std::sync::{
    atomic::Ordering,
    mpsc::{Receiver, RecvTimeoutError},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tauri::Emitter;
pub fn run(
    rx: Receiver<Command>,
    console: Arc<ConsoleEngine>,
    config: Arc<Mutex<AppConfig>>,
    snapshot: Arc<Mutex<Snapshot>>,
    app: tauri::AppHandle,
) {
    let controls: Vec<_> = BusId::PROGRAM_INPUTS
        .iter()
        .map(|b| console.duck_handle(*b))
        .collect();

    let mut state = MicState::default();
    let mut previous = None;
    let mut stream = None;
    let mut meter = Arc::new(Meter::default());
    let mut error = None;
    let mut preview = None;
    let mut preview_stream = None;
    let mut preview_meter = Arc::new(Meter::default());
    let mut preview_error = None;
    let mut preview_frame = 0;
    let mut preview_data = Instant::now();
    let mut last_frame = 0;
    let mut last_data = Instant::now();
    let mut last_tick = Instant::now();
    let mut last_emit = Instant::now();

    loop {
        let command = match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(Command::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(c) => Some(c),
            Err(RecvTimeoutError::Timeout) => None,
        };
        let (cfg, identity) = {
            let cfg = config.lock().unwrap();
            let audio = cfg.active_audio();
            (
                cfg.mic.clone(),
                (
                    cfg.active_profile_id.clone(),
                    audio.map(|a| a.key_mic.clone()).unwrap_or_default(),
                    audio.is_some_and(|a| a.global_keys),
                ),
            )
        };
        let current = (cfg.clone(), identity);
        let valid = cfg.validate().is_ok();
        if previous.as_ref() != Some(&current) {
            state = MicState::default();
            stream = None;
            error = cfg.validate().err();
            previous = Some(current);
        }
        match command {
            Some(Command::Preview(input)) => {
                preview_stream = None;
                (preview, preview_error) = capture::preview_input(input);
            }
            Some(Command::Input(source, pressed)) if valid => {
                if pressed {
                    error = None;
                }
                state.input(cfg.mode, source, pressed);
            }
            Some(Command::ReleaseLocal) => state.release_local(),
            Some(Command::Reset) => {
                state = MicState::default();
                error = None;
            }
            _ => (),
        }
        if cfg.mode == MicMode::Auto && state.listening {
            if stream.is_none() {
                if capture::input_name(&cfg.input).ok().as_ref() == preview.as_ref() {
                    preview_stream = None;
                }
                meter = Arc::new(Meter::default());
                last_frame = 0;
                last_data = Instant::now();
                match capture::open(&cfg.input, Arc::clone(&meter)) {
                    Ok(s) => stream = Some(s),
                    Err(e) => {
                        error = Some(e);
                        state.listening = false;
                    }
                }
            }
            let frame = meter.frames.load(Ordering::Acquire);
            if frame != last_frame {
                last_data = Instant::now();
                last_frame = frame;
            }
            if meter.failed.load(Ordering::Relaxed)
                || last_data.elapsed() > Duration::from_millis(500)
            {
                error = Some("mic.input_error".into());
                state.listening = false;
            }
        }
        if !state.listening {
            stream = None;
            state.clear_detector();
        }
        let db = if stream.is_some() {
            f32::from_bits(meter.db.load(Ordering::Relaxed))
        } else {
            -100.0
        };
        let elapsed = last_tick.elapsed().as_millis().min(50) as u64;
        last_tick = Instant::now();
        state.detect(db, elapsed, &cfg);
        let active = state.active(cfg.mode) && error.is_none();
        // Reutilizar la captura automática cuando mide la misma entrada.
        let shared = stream
            .as_ref()
            .is_some_and(|s: &capture::InputStream| s.matches(preview.as_ref()));
        if shared || preview.is_none() {
            preview_stream = None;
            if shared {
                preview_error = None;
            }
        } else if preview_stream.is_none() && preview_error.is_none() {
            preview_meter = Arc::new(Meter::default());
            preview_frame = 0;
            preview_data = Instant::now();
            match capture::open(preview.as_ref().unwrap(), Arc::clone(&preview_meter)) {
                Ok(s) => preview_stream = Some(s),
                Err(e) => preview_error = Some(e),
            }
        }
        let frame = preview_meter.frames.load(Ordering::Acquire);
        if frame != preview_frame {
            preview_frame = frame;
            preview_data = Instant::now();
        }
        if preview_stream.is_some()
            && (preview_meter.failed.load(Ordering::Relaxed)
                || preview_data.elapsed() > Duration::from_millis(500))
        {
            preview_stream = None;
            preview_error = Some("mic.input_error".into());
        }
        let preview_db = if shared {
            db
        } else if preview_stream.is_some() {
            f32::from_bits(preview_meter.db.load(Ordering::Relaxed))
        } else {
            -100.0
        };
        let parameters = if valid {
            cfg.clone()
        } else {
            Default::default()
        };
        for (control, selected) in controls.iter().zip([cfg.main, cfg.panel, cfg.player]) {
            control.set(
                active && selected && valid,
                parameters.level,
                parameters.down_ms,
                parameters.up_ms,
            );
        }
        let next = Snapshot {
            mode: cfg.mode,
            active,
            listening: state.listening,
            level_db: db,
            error: error.clone(),
            preview_level_db: preview_db,
            preview_error: preview_error.clone(),
        };
        *snapshot.lock().unwrap() = next.clone();
        if last_emit.elapsed() >= Duration::from_millis(100) {
            let _ = app.emit("mic-tick", next);
            last_emit = Instant::now();
        }
    }

    for control in controls {
        control.set(false, 1.0, 0, 0);
    }
}
