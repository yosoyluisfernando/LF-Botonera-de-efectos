//! Integración de las salidas por pestaña de Danilo Wenner (7c1f1ae), adaptada a la consola oficial.
use super::{device, BusId, ConsoleEngine};
use crate::{
    domain::console::tab_lanes::{self, OutputIssue, TabRoute},
    model::AppConfig,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

static APPLY: Mutex<()> = Mutex::new(());

pub fn lock_routes() -> std::sync::MutexGuard<'static, ()> { APPLY.lock().unwrap() }
#[derive(Clone, Default, PartialEq, Serialize)]
pub struct OutputStatus {
    pub devices: Vec<String>,
    pub issues: Vec<OutputIssue>,
}

pub fn apply(
    config: &Mutex<AppConfig>,
    console: &ConsoleEngine,
    retry: bool,
) -> Result<OutputStatus, String> {
    if cfg!(target_os = "android") {
        return Ok(OutputStatus::default());
    }
    let _gate = lock_routes();
    let descriptors = device::refresh_outputs();
    super::device_migration::apply(config, &descriptors)?;
    let devices = std::iter::once("default".into()).chain(descriptors.iter().map(|d| d.name.clone())).collect();
    let available: Vec<_> = descriptors.iter().map(|d| if d.id.is_empty() { d.name.clone() } else { d.id.clone() }).collect();
    let previous = console.tab_lanes();
    let cfg = config.lock().unwrap().clone();
    let Some(profile) = cfg.active_profile() else {
        return Ok(OutputStatus {
            devices,
            issues: vec![],
        });
    };
    let main = device::canonical_name(&profile.audio.out_main);
    let names: Vec<_> = profile
        .paletas
        .iter()
        .filter(|p| !p.obs)
        .map(|p| {
            (
                p,
                if p.audio_out.trim().is_empty() {
                    String::new()
                } else {
                    if p.audio_out_id.is_empty() { device::canonical_name(&p.audio_out) } else { p.audio_out_id.clone() }
                },
            )
        })
        .collect();
    let tabs: Vec<_> = names
        .iter()
        .map(|(p, name)| TabRoute {
            paleta_id: &p.id,
            nombre: &p.nombre,
            audio_out: name,
        })
        .collect();
    let plan = tab_lanes::plan(&tabs, &main, &available, &previous);
    let mapping = plan.paleta_bus.clone();
    let lanes = plan.lanes.clone();
    let mut issues = plan.issues.clone();
    // Inicializa solo al cambiar el dueño del carril; consultar estado no mueve faders.
    for (name, lane) in &lanes {
        if previous.get(name) != Some(lane) {
            console.set_fader(
                BusId::Salida(*lane),
                cfg.console
                    .output_faders
                    .get(name)
                    .copied()
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0),
            );
        }
    }
    console.apply_tab_routes(plan, available, retry)?;
    for tab in tabs {
        if let Some(bus) = mapping.get(tab.paleta_id) {
            if console.bus(*bus).is_none() && !issues.iter().any(|i| i.paleta_id == tab.paleta_id) {
                issues.push(OutputIssue {
                    paleta_id: tab.paleta_id.into(),
                    nombre: tab.nombre.into(),
                    device: tab.audio_out.into(),
                    reason: if device::descriptor(tab.audio_out).is_some_and(|d| !d.usable) { "ambiguous" } else { "unavailable" },
                });
            }
        }
    }
    for issue in &mut issues {
        if let Some(tab) = profile.paletas.iter().find(|p| p.id == issue.paleta_id) { issue.device = tab.audio_out.clone(); }
    }
    Ok(OutputStatus { devices, issues })
}

/// Independiente de las ventanas: también reconcilia importación, deshacer y perfiles.
pub fn start(app: tauri::AppHandle, config: Arc<Mutex<AppConfig>>, console: Arc<ConsoleEngine>) {
    std::thread::spawn(move || {
        let mut previous = None;
        loop {
            match apply(&config, &console, false) {
                Ok(status) => {
                    let current = (status.clone(), console.tab_lanes());
                    if previous.as_ref() != Some(&current) {
                        let _ = app.emit("tab-outputs-changed", &status);
                        previous = Some(current);
                    }
                }
                Err(e) => eprintln!("tab outputs: {e}"),
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            if app.get_webview_window("main").is_none() {
                break;
            }
        }
    });
}
