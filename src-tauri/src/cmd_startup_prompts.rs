use crate::{config, AppState};
use serde::Serialize;
use tauri::State;

const CHANGELOG: &str = include_str!("../../CHANGELOG.md");
const FIRST_DONATION_PROMPT: u32 = 5;
const DONATION_INTERVAL: u32 = 30;

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
pub fn prepare_startup_prompts(state: State<AppState>) -> Result<StartupPrompts, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.startup.launch_count = cfg.startup.launch_count.saturating_add(1);
    let notes = release_notes_for(&current);
    let show = cfg.startup.last_seen_version != current && !notes.is_empty();
    let donation_due = donation_due(cfg.startup.launch_count, cfg.startup.last_donation_prompt_launch);
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
pub fn mark_release_notes_seen(state: State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.startup.last_seen_version = env!("CARGO_PKG_VERSION").to_string();
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

fn release_notes_for(version: &str) -> String {
    let unreleased = section("[Sin publicar]");
    if !unreleased.trim().is_empty() {
        return unreleased;
    }
    section(&format!("[{version}]"))
}

fn section(marker: &str) -> String {
    let header = format!("## {marker}");
    let Some(start) = CHANGELOG.find(&header) else {
        return String::new();
    };
    let body = &CHANGELOG[start + header.len()..];
    let end = body.find("\n## ").unwrap_or(body.len());
    body[..end].trim_matches(|c| c == '\n' || c == '\r' || c == ' ').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donation_starts_on_fifth_launch_then_every_thirty() {
        assert!(!donation_due(4, 0));
        assert!(donation_due(5, 0));
        assert!(!donation_due(5, 5));
        assert!(!donation_due(30, 0));
        assert!(donation_due(35, 5));
        assert!(donation_due(65, 35));
    }

    #[test]
    fn extracts_unreleased_notes() {
        assert!(release_notes_for(env!("CARGO_PKG_VERSION")).contains("###"));
    }
}
