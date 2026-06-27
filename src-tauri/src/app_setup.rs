/// Módulo: app_setup.rs
/// Propósito: lógica de ARRANQUE de la app (dispositivo de audio, presupuesto de
/// precarga, hilos de monitor/reloj/historial/clima, recalentado de precarga y
/// flush al cerrar). Separado de lib.rs, que queda como manifiesto puro
/// (módulos, AppState y registro de comandos), sin lógica.
use crate::{
    audio_monitor, cmd_master_volume, cmd_meta, global_shortcuts, last_played, preload_warm,
    weather, AppState,
};
use std::sync::Arc;
use tauri::Manager;

/// Inicializa estado e hilos tras crear la ventana. Lo invoca `.setup` en lib.rs.
pub fn on_setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let cfg = state.config.lock().unwrap();
    let pid = cfg.active_profile_id.clone();
    let master_volume = cmd_master_volume::startup_volume(&cfg);
    let preload_budget = cfg.preload.ram_budget_mb;
    let device = cfg
        .profiles
        .iter()
        .find(|p| p.id == pid)
        .map(|p| p.audio.out_main.clone())
        .unwrap_or_else(|| "default".to_string());
    drop(cfg);

    let engine = state.audio.lock().unwrap();
    let _ = engine.set_device(&device);
    engine.set_master_volume(master_volume);
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
