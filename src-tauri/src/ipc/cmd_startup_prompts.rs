use crate::core::AppState;
use crate::engine::persist::config_io as config;
use serde::Serialize;
use tauri::State;

const FIRST_DONATION_PROMPT: u32 = 5;
const DONATION_INTERVAL: u32 = 100;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupPrompts {
    pub current_version: String,
    pub show_release_notes: bool,
    pub release_notes: String,
    pub donation_due: bool,
    pub launch_count: u32,
}

#[tauri::command]
pub fn prepare_startup_prompts(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<StartupPrompts, String> {
    let current = app.package_info().version.to_string();
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.startup.launch_count = cfg.startup.launch_count.saturating_add(1);
    let notes = crate::domain::release_notes::for_platform(&current, cfg!(target_os = "android"));
    let show = cfg.startup.last_seen_version != current && !notes.is_empty();
    let donation_due = donation_due(
        cfg.startup.launch_count,
        cfg.startup.last_donation_prompt_launch,
    );
    let launch_count = cfg.startup.launch_count;
    config::save_config(&cfg)?;
    Ok(StartupPrompts {
        current_version: current,
        show_release_notes: show,
        release_notes: notes,
        donation_due,
        launch_count,
    })
}

#[tauri::command]
pub fn mark_release_notes_seen(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.startup.last_seen_version = app.package_info().version.to_string();
    config::save_config(&cfg)
}

#[tauri::command]
pub fn mark_donation_prompt_shown(state: State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.startup.last_donation_prompt_launch = cfg.startup.launch_count;
    config::save_config(&cfg)
}

fn donation_due(launch_count: u32, last_prompt: u32) -> bool {
    launch_count >= FIRST_DONATION_PROMPT
        && (launch_count - FIRST_DONATION_PROMPT) % DONATION_INTERVAL == 0
        && last_prompt != launch_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donation_starts_on_fifth_launch_then_every_hundred() {
        assert!(!donation_due(4, 0));
        assert!(donation_due(5, 0));
        assert!(!donation_due(5, 5));
        assert!(!donation_due(35, 5));
        assert!(donation_due(105, 5));
        assert!(donation_due(205, 105));
    }
}
