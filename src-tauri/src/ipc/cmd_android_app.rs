#[tauri::command]
pub async fn android_back_attach(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    return crate::core::android_app::attach_back(&app).await;
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(())
    }
}

#[tauri::command]
pub async fn android_exit_app(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;
        let state = app.state::<crate::core::AppState>();
        state.audio.lock().unwrap().stop_all();
        state.player.lock().unwrap().stop();
        state.midi.cancel_capture();
        crate::engine::persist::last_played::flush_now(&state.last_played.handle(), &state.tracks);
        return crate::core::android_app::exit(&app).await;
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("platform_unavailable".into())
    }
}
