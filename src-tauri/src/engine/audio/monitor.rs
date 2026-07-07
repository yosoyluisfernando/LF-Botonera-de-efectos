/// Módulo: audio_monitor.rs
/// Propósito: Hilo de monitoreo que emite "audio-tick" cada ~100ms con el estado
/// de reproducción, tiempo restante para el reloj y niveles del vúmetro master.
/// Toda la lógica de cálculo vive en Rust (Regla 4).
use crate::engine::audio::bus::ButtonStateMap;
use crate::engine::audio::vu::LastPressedInfo;
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

#[derive(Serialize, Clone)]
pub struct TickInfo {
    pub id: String,
    pub pos: f64,
    pub remaining: f64,
    pub duration: f64,
}

#[derive(Serialize, Clone)]
pub struct AudioTickPayload {
    pub buttons: Vec<TickInfo>,
    /// Tiempo restante en segundos para el reloj de la barra inferior.
    pub display_remaining: f64,
    /// Duración original del audio que gobierna el contador de la barra inferior.
    pub display_duration: f64,
    pub master_level_l: f32,
    pub master_level_r: f32,
}

pub fn start(
    app: tauri::AppHandle,
    button_states: Arc<Mutex<ButtonStateMap>>,
    master_level_l: Arc<AtomicU32>,
    master_level_r: Arc<AtomicU32>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
) {
    thread::spawn(move || {
        let mut was_empty = false;
        loop {
            let (buttons, display_remaining, display_duration) = {
                let mut map = button_states.lock().unwrap();
                // Purgar botones terminados
                map.retain(|_, v| {
                    v.retain(|s| !s.is_done());
                    !v.is_empty()
                });

                let buttons: Vec<TickInfo> = map
                    .iter()
                    .filter_map(|(id, g)| {
                        g.iter().rev().find(|s| !s.is_done()).map(|s| TickInfo {
                            id: id.clone(),
                            pos: s.position(),
                            remaining: s.remaining(),
                            duration: s.duration,
                        })
                    })
                    .collect();
                let (rem, dur) = compute_display_time(&map, &last_pressed);
                (buttons, rem, dur)
            };

            let is_empty = buttons.is_empty();
            // Cuando no hay nada sonando, los atómicos aún pueden retener el último
            // pico medido (el LevelSource necesita ~21ms más para medir el silencio).
            // Forzar 0.0 aquí garantiza que el tick final lleve nivel cero y el
            // vúmetro anime correctamente hasta la base en lugar de quedar colgado.
            let (ml, mr) = if is_empty {
                (0.0f32, 0.0f32)
            } else {
                (
                    f32::from_bits(master_level_l.load(Ordering::Relaxed)),
                    f32::from_bits(master_level_r.load(Ordering::Relaxed)),
                )
            };
            if !is_empty || !was_empty {
                let _ = app.emit(
                    "audio-tick",
                    AudioTickPayload {
                        buttons,
                        display_remaining,
                        display_duration,
                        master_level_l: ml,
                        master_level_r: mr,
                    },
                );
            }
            was_empty = is_empty;
            thread::sleep(Duration::from_millis(100));
        }
    });
}

/// Calcula el tiempo restante a mostrar en el reloj de la barra inferior.
/// Regla: muestra el del último botón presionado; si éste terminó, el de mayor tiempo restante.
fn compute_display_time(
    map: &ButtonStateMap,
    last_pressed: &Mutex<Option<LastPressedInfo>>,
) -> (f64, f64) {
    let lp = last_pressed.lock().unwrap();
    let Some(info) = lp.as_ref() else {
        return (0.0, 0.0);
    };

    // Si el último botón presionado aún está sonando, retornar su tiempo restante
    if let Some(states) = map.get(&info.id) {
        if let Some(s) = states.iter().rev().find(|s| !s.is_done()) {
            let rem = s.remaining();
            if rem > 0.0 {
                return (rem, s.duration);
            }
        }
    }

    // Último presionado terminó: retornar el mayor tiempo restante de los activos
    map.values()
        .flat_map(|states| states.iter().filter(|s| !s.is_done()))
        .map(|s| (s.remaining(), s.duration))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap_or((0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::compute_display_time;
    use crate::engine::audio::bus::{ButtonState, ButtonStateMap};
    use crate::engine::audio::vu::LastPressedInfo;
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
}
