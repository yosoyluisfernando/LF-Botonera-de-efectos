/// Módulo: cmd_updates.rs
/// Propósito: Revisión segura de nuevas versiones publicadas en GitHub Releases.
/// La UI no consulta GitHub directamente: Rust controla cadencia y comparación.
use crate::{config, AppState};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;

const CHECK_INTERVAL_SECS: i64 = 12 * 60 * 60;
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/yosoyluisfernando/LF-Botonera-de-efectos/releases/latest";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub checked: bool,
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub notes: String,
}

/// Revisa GitHub Releases. Si `force` es falso, respeta una ventana de 12 horas.
#[tauri::command]
pub fn check_for_updates(state: State<AppState>, force: bool) -> Result<UpdateCheck, String> {
    let now = unix_now();
    if !force {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if now - cfg.last_update_check < CHECK_INTERVAL_SECS {
            return Ok(no_check());
        }
    }

    let release = fetch_latest_release()?;
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.last_update_check = now;
        config::save_config(&cfg)?;
    }

    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = normalize_version(&release.tag_name);
    Ok(UpdateCheck {
        checked: true,
        update_available: is_newer(&latest, &current),
        current_version: current,
        latest_version: latest,
        release_url: release.html_url,
        notes: release.body.unwrap_or_default(),
    })
}

fn fetch_latest_release() -> Result<GithubRelease, String> {
    ureq::get(LATEST_RELEASE_URL)
        .set("User-Agent", "LF-Botonera")
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())
}

fn no_check() -> UpdateCheck {
    UpdateCheck {
        checked: false,
        update_available: false,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        latest_version: String::new(),
        release_url: String::new(),
        notes: String::new(),
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn normalize_version(value: &str) -> String {
    value.trim().trim_start_matches('v').to_string()
}

fn is_newer(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

fn parse_version(value: &str) -> Vec<u32> {
    normalize_version(value)
        .split('.')
        .map(|p| p.parse::<u32>().unwrap_or(0))
        .collect()
}
