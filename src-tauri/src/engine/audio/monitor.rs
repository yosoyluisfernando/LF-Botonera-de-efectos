/// Módulo: audio_monitor.rs
/// Propósito: Hilo de monitoreo que emite "audio-tick" cada ~100ms con el estado
/// de reproducción, tiempo restante para el reloj y el nivel de cada bus.
/// Toda la lógica de cálculo vive en Rust (Regla 4).
/// Lo que viaja en el tick está en `tick.rs`.
use crate::engine::audio::button::{ButtonStateMap, PlaybackGroup};
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::audio::tick::{AudioTickPayload, LevelTaps, TickInfo};
use crate::engine::console::ConsoleEngine;
use crate::engine::player::PlayerSnapshot;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

pub fn start(
    app: tauri::AppHandle,
    button_states: Arc<Mutex<ButtonStateMap>>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    player: Arc<Mutex<PlayerSnapshot>>,
    console: Arc<ConsoleEngine>,
) {
    thread::spawn(move || {
        // Una vez y para siempre: los atómicos son del BusSlot y sobreviven a que
        // el grafo se rehaga.
        let taps = LevelTaps::new(&console);
        let mut was_idle = false;
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
                        g.iter().rev().find(|s| !s.is_done()).map(|s| {
                            let remaining = s.remaining();
                            TickInfo {
                                id: id.clone(),
                                pos: s.position(),
                                remaining,
                                duration: s.duration,
                                group: match s.group {
                                    PlaybackGroup::Main => "main",
                                    PlaybackGroup::Fixed => "fixed",
                                },
                                progress_percent: if s.duration > 0.0 {
                                    (remaining / s.duration * 100.0).clamp(0.0, 100.0)
                                } else {
                                    0.0
                                },
                            }
                        })
                    })
                    .collect();
                let (rem, dur) = compute_display_time(&map, &last_pressed);
                (buttons, rem, dur)
            };

            // El vúmetro mide el bus Programa, y ahí suman los efectos, el panel
            // Y el reproductor. Así que "no hay nada sonando" no es "no hay
            // botones": con música de fondo y sin efectos hay señal de sobra, y
            // callar aquí dejaría la aguja plana mientras suena la música.
            let idle = buttons.is_empty() && !player.lock().unwrap().playing;
            // En reposo los atómicos aún pueden retener el último pico medido (el
            // LevelSource necesita ~21ms más para medir el silencio). Forzar 0.0
            // garantiza que el tick final lleve nivel cero y el vúmetro baje hasta
            // la base en lugar de quedarse colgado.
            let (ml, mr) = if idle { (0.0, 0.0) } else { taps.program() };
            if !idle || !was_idle {
                let _ = app.emit(
                    "audio-tick",
                    AudioTickPayload {
                        buttons,
                        display_remaining,
                        display_duration,
                        master_level_l: ml,
                        master_level_r: mr,
                        buses: taps.buses(idle),
                        idle,
                    },
                );
            }
            was_idle = idle;
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
        .flat_map(|states| {
            states
                .iter()
                .filter(|s| !s.is_done() && s.group == PlaybackGroup::Main)
        })
        .map(|s| (s.remaining(), s.duration))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap_or((0.0, 0.0))
}


#[cfg(test)]
#[path = "monitor_tests.rs"]
mod monitor_tests;
