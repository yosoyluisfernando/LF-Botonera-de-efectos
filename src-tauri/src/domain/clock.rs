use crate::model::AppConfig;
use chrono::{Datelike, Local};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

#[derive(Serialize, Clone)]
pub struct ClockTickPayload {
    pub time_str: String,
    pub date_str: String,
    /// true = 24 h activo, false = 12 h. Lo lee el JS para marcar el menu contextual.
    pub clock_24h: bool,
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

/// Formatea la fecha segun el idioma activo. Rust es el unico formateador.
fn format_date(lang: &str, weekday: usize, day: usize, month0: usize, year: u32) -> String {
    const DAYS_ES: [&str; 7] = [
        "Domingo", "Lunes", "Martes", "Miercoles", "Jueves", "Viernes", "Sabado",
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
    const DAYS_PT: [&str; 7] = [
        "Domingo",
        "Segunda-feira",
        "Terca-feira",
        "Quarta-feira",
        "Quinta-feira",
        "Sexta-feira",
        "Sabado",
    ];
    const MONTHS_PT: [&str; 12] = [
        "Janeiro",
        "Fevereiro",
        "Marco",
        "Abril",
        "Maio",
        "Junho",
        "Julho",
        "Agosto",
        "Setembro",
        "Outubro",
        "Novembro",
        "Dezembro",
    ];
    if lang == "en" {
        format!(
            "{}, {} {}, {}",
            DAYS_EN[weekday.min(6)],
            MONTHS_EN[month0.min(11)],
            day,
            year
        )
    } else if lang.starts_with("pt") {
        format!(
            "{}, {} de {} de {}",
            DAYS_PT[weekday.min(6)],
            day,
            MONTHS_PT[month0.min(11)],
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
