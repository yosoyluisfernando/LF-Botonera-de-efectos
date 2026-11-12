use crate::core::AppState;
use crate::engine::persist::config_io;
use serde::Serialize;

#[derive(Serialize)]
pub struct ShortcutEditorState {
    keyboard_available: bool,
    midi_available: bool,
}

#[tauri::command]
pub async fn get_shortcut_editor_state(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ShortcutEditorState, String> {
    let midi_available = {
        let cfg = state.config.lock().map_err(|error| error.to_string())?;
        cfg.midi.enabled
            && state
                .midi
                .devices(&cfg.midi)
                .iter()
                .any(|device| device.connected)
    };
    #[cfg(target_os = "android")]
    let keyboard_available = !crate::engine::input::midi_android::keyboards(&app)
        .await?
        .is_empty();
    #[cfg(not(target_os = "android"))]
    let keyboard_available = {
        let _ = app;
        true
    };
    Ok(ShortcutEditorState {
        keyboard_available,
        midi_available,
    })
}

#[tauri::command]
pub fn set_shortcut_label_visibility(
    keyboard: bool,
    midi: bool,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|error| error.to_string())?;
    let mut next = cfg.clone();
    next.show_keyboard_shortcut_labels = keyboard;
    next.show_midi_shortcut_labels = midi;
    config_io::save_config(&next)?;
    *cfg = next;
    Ok(())
}
