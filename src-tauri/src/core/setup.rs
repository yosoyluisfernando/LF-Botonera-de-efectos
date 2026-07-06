/// Modulo: core/setup.rs
/// Proposito: logica de ARRANQUE de la app (dispositivo de audio, presupuesto de
/// precarga, hilos de monitor/reloj/historial/clima, recalentado de precarga y
/// flush al cerrar). Separado de lib.rs, que queda como manifiesto puro
/// (modulos, AppState y registro de comandos), sin logica.
use crate::engine::audio::monitor as audio_monitor;
use crate::engine::cache::warm as preload_warm;
use crate::engine::input::keyboard as global_shortcuts;
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
    drop(cfg);

    let engine = state.audio.lock().unwrap();
    let _ = engine.set_device(&device);
    engine.set_master_volume(master_volume);
    engine.set_preload_enabled(preload_enabled);
    // Pre-escucha: solo si es una tarjeta distinta de la principal (fallback).
    let pre = if out_pre.is_empty() || out_pre == device { "" } else { &out_pre };
    let _ = engine.set_pre_device(pre);
    engine
        .preload_cache_handle()
        .lock()
        .unwrap()
        .set_budget(preload_budget);

    // Hilo monitor: emite "audio-tick" con progreso, tiempo restante y niveles VU
    let (ll, lr) = engine.master_levels_handles();
    audio_monitor::start(
        app.handle().clone(),
        engine.button_states_handle(),
        ll,
        lr,
        engine.last_pressed_handle(),
    );
    drop(engine);

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
