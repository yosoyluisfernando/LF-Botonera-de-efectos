/// Modulo: core/setup.rs
/// Proposito: logica de ARRANQUE de la app (dispositivo de audio, presupuesto de
/// precarga, hilos de monitor/reloj/historial/clima, recalentado de precarga y
/// flush al cerrar). Separado de lib.rs, que queda como manifiesto puro
/// (modulos, AppState y registro de comandos), sin logica.
use crate::engine::audio::monitor as audio_monitor;
use crate::engine::cache::warm as preload_warm;
use crate::engine::console::BusId;
use crate::engine::input::keyboard as global_shortcuts;
use crate::engine::player::monitor as player_monitor;
use crate::engine::persist::last_played;
use crate::engine::weather::client as weather;
use crate::ipc::{cmd_master_volume, cmd_meta};
use crate::core::AppState;
use std::sync::Arc;
use tauri::Manager;

/// Inicializa estado e hilos tras crear la ventana. Lo invoca `.setup` en lib.rs.
pub fn on_setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let cfg = state.config.lock().unwrap();
    let master_volume = cmd_master_volume::startup_volume(&cfg);
    let preload_budget = cfg.preload.ram_budget_mb;
    let preload_enabled = cfg.preload.enabled;
    let profile_audio = cfg.active_audio();
    let device = profile_audio
        .map(|a| a.out_main.clone())
        .unwrap_or_else(|| "default".to_string());
    let out_pre = profile_audio.map(|a| a.out_pre.clone()).unwrap_or_default();
    // Reproductor auxiliar: "" = mismo dispositivo que los efectos.
    let player_device_cfg = cfg.player.output_device.clone();
    let player_volume = cfg.player.volume;
    drop(cfg);

    let engine = state.audio.lock().unwrap();
    let _ = engine.set_device(&device);
    // El master es el fader del bus Programa. Se pone antes de que el bus exista
    // a proposito: el atomico vive en el slot desde que nace la consola, y el bus
    // lo toma al abrirse. Asi no hay carrera con el `set_device` de arriba, que
    // es asincrono.
    state.console.set_fader(BusId::Programa, master_volume);
    engine.set_preload_enabled(preload_enabled);
    // Pre-escucha: vacio = comparte la tarjeta del programa. No es un fallback —
    // sigue siendo un bus aparte, sin master y fuera del vumetro de programa.
    let pre = if out_pre.is_empty() || out_pre == device { "" } else { &out_pre };
    let _ = engine.set_pre_device(pre);
    engine
        .preload_cache_handle()
        .lock()
        .unwrap()
        .set_budget(preload_budget);

    // Hilo monitor: emite "audio-tick" con progreso, tiempo restante y niveles VU.
    // El vumetro mide el bus Programa: el mismo que gobierna el master, que es la
    // unica forma de que la aguja no mienta sobre lo que el fader controla.
    let (ll, lr) = state.console.levels(BusId::Programa);
    audio_monitor::start(
        app.handle().clone(),
        engine.button_states_handle(),
        ll,
        lr,
        engine.last_pressed_handle(),
    );
    drop(engine);

    // Motor propio del reproductor auxiliar: dispositivo y volumen propios. Si
    // el dispositivo configurado esta vacio, sale por el mismo de los efectos.
    let player_device = if player_device_cfg.is_empty() {
        device.clone()
    } else {
        player_device_cfg
    };
    let player_snapshot = {
        let player = state.player.lock().unwrap();
        player.set_device(&player_device);
        player.set_volume(player_volume);
        player.snapshot_handle()
    };
    // Sincroniza el modo y la cola guardados con el motor del reproductor.
    crate::ipc::cmd_player::apply_startup(&state);

    // Hilo monitor del reproductor: emite "player-tick". Es propio porque
    // "audio-tick" no se emite en reposo y la musica suena sin efectos.
    player_monitor::start(app.handle().clone(), player_snapshot);

    // Hilo del reloj: emite "clock-tick" con hora y fecha localizadas
    cmd_meta::start_clock_thread(app.handle().clone(), Arc::clone(&state.config));

    // Hilo que vuelca el historial de reproducción a tracks.db (debounce)
    last_played::start_flusher(state.last_played.handle(), Arc::clone(&state.tracks));

    // Precarga proactiva según la estrategia (perfil completo / pestaña visible)
    preload_warm::warm_for_strategy(&state);
    // Recalentado OnPlay: precarga lo reproducido recientemente (TTL)
    preload_warm::warm_onplay_recent(&state);

    // Volcar el historial pendiente al cerrar la ventana (además del debounce)
    if let Some(win) = app.get_webview_window("main") {
        let pending = state.last_played.handle();
        let store = Arc::clone(&state.tracks);
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                last_played::flush_now(&pending, &store);
            }
        });
    }

    // Hilo de clima: refresca cada 15 min y emite "weather-updated"
    weather::start_auto_refresh(app.handle().clone());
    let _ = global_shortcuts::sync(app.handle());
    Ok(())
}
