/// Módulo: cmd_meta.rs
/// Propósito: Metadatos de la aplicación (versión) y el hilo del reloj.
/// El formato de fecha/hora se realiza en Rust para cumplir Regla 4 y Regla 6.
use crate::config::save_config;
use crate::types::AppConfig;
use crate::AppState;
use chrono::{Datelike, Local};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, State};

#[derive(Serialize, Clone)]
pub struct ClockTickPayload {
    pub time_str: String,
    pub date_str: String,
    /// true = 24 h activo, false = 12 h. Lo lee el JS para marcar el menú contextual.
    pub clock_24h: bool,
}

/// Devuelve la versión del ejecutable compilada desde Cargo.toml (Regla 6).
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Alterna entre formato 24 h y 12 h (sin AM/PM). Persiste el cambio en disco.
/// El hilo del reloj lo lee en el siguiente tick sin necesitar reinicio.
#[tauri::command]
pub fn toggle_clock_format(state: State<AppState>) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.clock_24h = !cfg.clock_24h;
    let new_val = cfg.clock_24h;
    let _ = save_config(&cfg);
    new_val
}

/// Arranca el hilo del reloj que emite "clock-tick" cada segundo.
/// Lee idioma y formato en cada tick para reaccionar a cambios sin reinicio.
pub fn start_clock_thread(app: tauri::AppHandle, config: Arc<Mutex<AppConfig>>) {
    thread::spawn(move || loop {
        let (lang, clock_24h) = {
            let cfg = config.lock().unwrap();
            (cfg.language.clone(), cfg.clock_24h)
        };
        let now = Local::now();
        let time_str = if clock_24h {
            now.format("%H:%M:%S").to_string()
        } else {
            now.format("%I:%M:%S").to_string()
        };
        let date_str = format_date(
            &lang,
            now.weekday().num_days_from_sunday() as usize,
            now.day() as usize,
            now.month0() as usize,
            now.year() as u32,
        );
        let _ = app.emit(
            "clock-tick",
            ClockTickPayload {
                time_str,
                date_str,
                clock_24h,
            },
        );
        thread::sleep(Duration::from_millis(1000));
    });
}

/// Formatea la fecha según el idioma activo. Rust es el único formateador (Regla 4 + Regla 6).
fn format_date(lang: &str, weekday: usize, day: usize, month0: usize, year: u32) -> String {
    const DAYS_ES: [&str; 7] = [
        "Domingo",
        "Lunes",
        "Martes",
        "Miércoles",
        "Jueves",
        "Viernes",
        "Sábado",
    ];
    const MONTHS_ES: [&str; 12] = [
        "Enero",
        "Febrero",
        "Marzo",
        "Abril",
        "Mayo",
        "Junio",
        "Julio",
        "Agosto",
        "Septiembre",
        "Octubre",
        "Noviembre",
        "Diciembre",
    ];
    const DAYS_EN: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const MONTHS_EN: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    if lang == "en" {
        format!(
            "{}, {} {}, {}",
            DAYS_EN[weekday.min(6)],
            MONTHS_EN[month0.min(11)],
            day,
            year
        )
    } else {
        format!(
            "{}, {} de {} {}",
            DAYS_ES[weekday.min(6)],
            day,
            MONTHS_ES[month0.min(11)],
            year
        )
    }
}
