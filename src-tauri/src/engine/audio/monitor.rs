/// Módulo: audio_monitor.rs
/// Propósito: Hilo de monitoreo que emite "audio-tick" cada ~100ms con el estado
/// de reproducción, tiempo restante para el reloj y niveles del vúmetro master.
/// Toda la lógica de cálculo vive en Rust (Regla 4).
use crate::engine::audio::button::{ButtonStateMap, PlaybackGroup};
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::player::PlayerSnapshot;
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
    pub group: &'static str,
    pub progress_percent: f64,
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
    /// Ya no suena nada en el programa: ni efectos, ni panel, ni reproductor.
    /// Este es el ÚLTIMO tick antes del silencio, y va con nivel cero.
    ///
    /// Lo decide Rust porque solo Rust lo sabe (regla 4): el frontend lo deducía
    /// de que la lista de botones viniera vacía, y con música de fondo sin
    /// efectos eso es falso — hay señal de sobra. El vúmetro daba entonces cada
    /// tick por final y le ponía el decaimiento largo, así que la aguja nunca
    /// alcanzaba el nivel real.
    pub idle: bool,
}

pub fn start(
    app: tauri::AppHandle,
    button_states: Arc<Mutex<ButtonStateMap>>,
    master_level_l: Arc<AtomicU32>,
    master_level_r: Arc<AtomicU32>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    player: Arc<Mutex<PlayerSnapshot>>,
) {
    thread::spawn(move || {
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
            let (ml, mr) = if idle {
                (0.0f32, 0.0f32)
            } else {
                (
                    f32::from_bits(master_level_l.load(Ordering::Relaxed)),
                    f32::from_bits(master_level_r.load(Ordering::Relaxed)),
                )
            };
            if !idle || !was_idle {
                let _ = app.emit(
                    "audio-tick",
                    AudioTickPayload {
                        buttons,
                        display_remaining,
                        display_duration,
                        master_level_l: ml,
                        master_level_r: mr,
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
