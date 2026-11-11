#[tauri::command]
pub async fn keyboard_devices(app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(target_os = "android")]
    return crate::engine::input::midi_android::keyboards(&app).await;
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(Vec::new())
    }
}

#[tauri::command]
pub async fn keyboard_open_bluetooth_settings(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    return crate::engine::input::midi_android::open_keyboard_settings(&app).await;
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("platform_unavailable".into())
    }
}
