/// Modulo: cmd_console.rs
/// Proposito: comandos IPC de la consola de audio. La UI pinta tiras de canal y
/// mueve faders; todo lo que hay detras lo decide el motor.
use super::AppState;
use crate::engine::console::BusId;
use crate::engine::persist::config_io as config;
use serde::Serialize;

/// Una tira de canal, tal cual la necesita la UI para pintarse.
#[derive(Serialize)]
pub struct StripView {
    /// Identificador estable para el IPC. La ETIQUETA la pone el frontend por
    /// i18n: aqui no se escriben textos visibles (regla 7).
    pub bus: &'static str,
    pub fader: f32,
    /// El techo del fader. El programa llega a 1.5 en modo boost; los demas a 1.0.
    pub max: f32,
    /// Si suma en el programa. Un bus fuera del programa no obedece al master, y
    /// la tira lo tiene que decir.
    pub in_program: bool,
}

#[derive(Serialize)]
pub struct ConsoleView {
    /// Los buses que suman en el programa, en orden de tira.
    pub strips: Vec<StripView>,
    /// El programa: su fader es el master.
    pub program: StripView,
    /// La pre-escucha. Va aparte porque **no suma en el programa**, y esa es la
    /// idea entera de la consola: la tira debe verse separada.
    pub cue: StripView,
    /// "window" | "modal".
    pub mode: String,
}

#[tauri::command]
pub fn get_console_view(state: tauri::State<AppState>) -> Result<ConsoleView, String> {
    let cfg = state.config.lock().unwrap();
    let audio = cfg.active_audio().ok_or("active_profile_not_found")?;
    let boost = audio.master_volume_boost;
    let console = &state.console;
    Ok(ConsoleView {
        strips: vec![
            strip("efectos", console.fader(BusId::Efectos), 1.0, true),
            strip("panel", console.fader(BusId::Panel), 1.0, true),
            strip(
                "reproductor",
                console.fader(BusId::Reproductor),
                1.5,
                cfg.player.output_device.is_empty(),
            ),
        ],
        program: strip(
            "programa",
            console.fader(BusId::Programa),
            if boost { 1.5 } else { 1.0 },
            true,
        ),
        cue: strip("cue", console.fader(BusId::Cue), 1.0, false),
        mode: cfg.console_mode.clone(),
    })
}

/// Mueve el fader de un bus.
///
/// `persist: false` mientras se arrastra: aplicar es un atomico, pero guardar en
/// cada pixel seria una tormenta de escrituras. Al soltar se llama con `true`. Es
/// el mismo cuidado que ya tenian el master y el volumen del reproductor.
#[tauri::command]
pub fn set_bus_fader(
    bus: String,
    value: f32,
    persist: Option<bool>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let id = bus_of(&bus)?;
    // El master y el volumen del reproductor tienen dueño desde antes de que
    // existiera la consola, con sus reglas propias (el "recordar" del master, el
    // modo boost). Se delega en ellos en vez de duplicarlos aqui.
    match id {
        BusId::Programa => {
            super::cmd_master_volume::set_master_volume(value, state)?;
            return Ok(());
        }
        BusId::Reproductor => {
            return super::cmd_player_config::player_set_volume(value, persist, state);
        }
        _ => {}
    }
    state.console.set_fader(id, value.clamp(0.0, 1.0));
    if persist.unwrap_or(true) {
        let mut cfg = state.config.lock().unwrap();
        let slot = match id {
            BusId::Efectos => &mut cfg.console.efectos,
            BusId::Panel => &mut cfg.console.panel,
            _ => &mut cfg.console.cue,
        };
        *slot = value.clamp(0.0, 1.0);
        config::save_config(&cfg)?;
    }
    Ok(())
}

/// Como se abre la consola: "window" (flotante) | "modal". Se recuerda.
#[tauri::command]
pub fn set_console_mode(mode: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.console_mode = if mode == "window" { mode } else { "modal".into() };
    config::save_config(&cfg)
}

fn strip(bus: &'static str, fader: f32, max: f32, in_program: bool) -> StripView {
    StripView {
        bus,
        fader,
        max,
        in_program,
    }
}

fn bus_of(name: &str) -> Result<BusId, String> {
    match name {
        "efectos" => Ok(BusId::Efectos),
        "panel" => Ok(BusId::Panel),
        "reproductor" => Ok(BusId::Reproductor),
        "cue" => Ok(BusId::Cue),
        "programa" => Ok(BusId::Programa),
        _ => Err("unknown_bus".to_string()),
    }
}
