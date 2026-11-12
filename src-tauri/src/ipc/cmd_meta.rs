/// Modulo: cmd_meta.rs
/// Proposito: comandos IPC de metadatos de la aplicacion.
use crate::core::AppState;
use crate::domain::clock;
use crate::domain::distribution;
use crate::engine::persist::config_io::save_config;
use serde::Serialize;
use tauri::State;

pub use clock::start_clock_thread;

/// Versión efectiva del paquete; respeta el override de cada plataforma.
#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionInfo {
    version: String,
    channel: &'static str,
    platform: &'static str,
    update_manager: &'static str,
}

/// Metadatos del paquete y del canal responsable de sus actualizaciones.
#[tauri::command]
pub async fn get_distribution_info(app: tauri::AppHandle) -> Result<DistributionInfo, String> {
    let channel = distribution_channel(&app).await?;
    Ok(DistributionInfo {
        version: app.package_info().version.to_string(),
        channel: channel.code(),
        platform: distribution::current_platform(),
        update_manager: channel.update_manager(),
    })
}

pub async fn distribution_channel(
    app: &tauri::AppHandle,
) -> Result<distribution::DistributionChannel, String> {
    #[cfg(target_os = "android")]
    {
        let installer = crate::core::android_app::installer_source(app).await?;
        Ok(distribution::android_channel(
            &installer,
            option_env!("LF_DISTRIBUTION_CHANNEL"),
        ))
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(distribution::current_channel())
    }
}

/// Alterna entre formato 24 h y 12 h. Persiste el cambio en disco.
#[tauri::command]
pub fn toggle_clock_format(state: State<AppState>) -> bool {
    let mut cfg = state.config.lock().unwrap();
    cfg.clock_24h = !cfg.clock_24h;
    let new_val = cfg.clock_24h;
    let _ = save_config(&cfg);
    new_val
}
