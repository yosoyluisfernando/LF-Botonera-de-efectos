//! Revisión de GitHub por plataforma; las tiendas administran sus propias versiones.
use crate::core::AppState;
use crate::domain::{distribution, update_release};
use crate::engine::persist::config_io as config;
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;

const CHECK_INTERVAL_SECS: i64 = 12 * 60 * 60;
const RELEASES_URL: &str =
    "https://api.github.com/repos/yosoyluisfernando/LF-Botonera-de-efectos/releases?per_page=100";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub checked: bool,
    pub store_managed: bool,
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub notes: String,
}

#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    force: bool,
    startup: Option<bool>,
) -> Result<UpdateCheck, String> {
    let current = app.package_info().version.to_string();
    let channel = super::cmd_meta::distribution_channel(&app).await?;
    if channel.managed_externally() {
        return Ok(empty_check(current, false, true));
    }
    let now = unix_now();
    if !force && !startup.unwrap_or(false) {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if now - cfg.last_update_check < CHECK_INTERVAL_SECS {
            return Ok(empty_check(current, false, false));
        }
    }
    let release = tauri::async_runtime::spawn_blocking(fetch_latest_release)
        .await
        .map_err(|error| error.to_string())??;
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.last_update_check = now;
        config::save_config(&cfg)?;
    }
    let Some(release) = release else {
        return Ok(empty_check(current, true, false));
    };
    let latest = update_release::normalize_version(&release.tag_name).to_string();
    Ok(UpdateCheck {
        checked: true,
        store_managed: false,
        update_available: update_release::is_newer(&latest, &current),
        current_version: current,
        latest_version: latest,
        release_url: release.html_url,
        notes: release.body.unwrap_or_default(),
    })
}

fn fetch_latest_release() -> Result<Option<update_release::GithubRelease>, String> {
    let releases = ureq::get(RELEASES_URL)
        .set("User-Agent", "LF-Botonera")
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|error| error.to_string())?
        .into_json()
        .map_err(|error| error.to_string())?;
    Ok(update_release::latest_for(
        releases,
        distribution::current_platform(),
    ))
}

fn empty_check(current: String, checked: bool, store_managed: bool) -> UpdateCheck {
    UpdateCheck {
        checked,
        store_managed,
        update_available: false,
        current_version: current,
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
