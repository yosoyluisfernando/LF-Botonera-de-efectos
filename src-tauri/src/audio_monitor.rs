/// Módulo: audio_monitor.rs
/// Propósito: Hilo de monitoreo que emite "audio-tick" cada ~100ms con el estado
/// de reproducción, tiempo restante para el reloj y niveles del vúmetro master.
/// Toda la lógica de cálculo vive en Rust (Regla 4).

use crate::master_bus::ButtonStateMap;
use crate::vu_meter::LastPressedInfo;
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

#[derive(Serialize, Clone)]
pub struct TickInfo {
    pub id:  String,
    pub pos: f64,
}

#[derive(Serialize, Clone)]
pub struct AudioTickPayload {
    pub buttons:           Vec<TickInfo>,
    /// Tiempo restante en segundos para el reloj de la barra inferior.
    pub display_remaining: f64,
    pub master_level_l:    f32,
    pub master_level_r:    f32,
}

pub fn start(
    app:            tauri::AppHandle,
    button_states:  Arc<Mutex<ButtonStateMap>>,
    master_level_l: Arc<AtomicU32>,
    master_level_r: Arc<AtomicU32>,
    last_pressed:   Arc<Mutex<Option<LastPressedInfo>>>,
) {
    thread::spawn(move || {
        let mut was_empty = false;
        loop {
            let (buttons, display_remaining) = {
                let mut map = button_states.lock().unwrap();
                // Purgar botones terminados
                map.retain(|_, v| { v.retain(|s| !s.is_done()); !v.is_empty() });

                let buttons: Vec<TickInfo> = map.iter()
                    .filter_map(|(id, g)| g.first().map(|s| TickInfo {
                        id: id.clone(), pos: s.position(),
                    }))
                    .collect();
                let rem = compute_display_remaining(&map, &last_pressed);
                (buttons, rem)
            };

            let is_empty = buttons.is_empty();
            // Cuando no hay nada sonando, los atómicos aún pueden retener el último
            // pico medido (el LevelSource necesita ~21ms más para medir el silencio).
            // Forzar 0.0 aquí garantiza que el tick final lleve nivel cero y el
            // vúmetro anime correctamente hasta la base en lugar de quedar colgado.
            let (ml, mr) = if is_empty {
                (0.0f32, 0.0f32)
            } else {
                (f32::from_bits(master_level_l.load(Ordering::Relaxed)),
                 f32::from_bits(master_level_r.load(Ordering::Relaxed)))
            };
            if !is_empty || !was_empty {
                let _ = app.emit("audio-tick", AudioTickPayload {
                    buttons, display_remaining, master_level_l: ml, master_level_r: mr,
                });
            }
            was_empty = is_empty;
            thread::sleep(Duration::from_millis(100));
        }
    });
}

/// Calcula el tiempo restante a mostrar en el reloj de la barra inferior.
/// Regla: muestra el del último botón presionado; si éste terminó, el de mayor tiempo restante.
fn compute_display_remaining(
    map:          &ButtonStateMap,
    last_pressed: &Mutex<Option<LastPressedInfo>>,
) -> f64 {
    let lp = last_pressed.lock().unwrap();
    let Some(info) = lp.as_ref() else { return 0.0; };

    // Si el último botón presionado aún está sonando, retornar su tiempo restante
    if let Some(states) = map.get(&info.id) {
        if let Some(s) = states.first() {
            let rem = s.remaining();
            if rem > 0.0 { return rem; }
        }
    }

    // Último presionado terminó: retornar el mayor tiempo restante de los activos
    map.values()
        .filter_map(|states| states.first())
        .map(|s| s.remaining())
        .fold(0.0f64, f64::max)
}
