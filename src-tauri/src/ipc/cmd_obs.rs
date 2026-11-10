//! Puertas IPC de OBS. El motor posee la red; el dominio valida los datos.
use super::AppState;
use crate::{
    domain::{button::defaults::new_button, obs},
    engine::{
        input::{keyboard, midi_rules, rules},
        persist::config_io,
    },
    model::{
        obs::{ObsAction, ObsConfig},
        ButtonVisual, MidiBinding,
    },
};
use obs::view::ObsView;

#[tauri::command]
pub fn obs_get(state: tauri::State<AppState>) -> ObsView {
    let snapshot = state.obs.snapshot();
    obs::view::view(&state.config.lock().unwrap(), snapshot)
}
#[tauri::command]
pub fn obs_set_config(
    config: ObsConfig,
    password: Option<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if cfg!(target_os = "android") { return Err("platform_unavailable".into()); }
    obs::validate_config(&config)?;
    let previous = state.config.lock().unwrap().obs.clone();
    if previous == config && password.is_none() {
        return Ok(());
    }
    {
        let mut cfg = state.config.lock().unwrap();
        let mut next = cfg.clone();
        next.obs = config.clone();
        config_io::save_config(&next)?;
        *cfg = next;
    }
    if obs::connection_changed(&previous, &config) || password.is_some() {
        state.obs.configure(config, password)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn obs_test(
    config: ObsConfig,
    password: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if cfg!(target_os = "android") { return Err("platform_unavailable".into()); }
    obs::validate_config(&config)?;
    let password = password.unwrap_or_else(|| state.obs.session_password());
    tauri::async_runtime::spawn_blocking(move || {
        crate::engine::obs::transport::Session::connect(&config, &password).map(|_| ())
    })
    .await
    .map_err(|_| "obs_worker_failed".to_string())?
}
#[tauri::command]
pub fn obs_save_button(
    index: u32,
    label: String,
    color_bg: String,
    color_text: String,
    action: ObsAction,
    shortcut: String,
    midi: MidiBinding,
    visual: ButtonVisual,
    replace_shortcut: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if cfg!(target_os = "android") { return Err("platform_unavailable".into()); }
    obs::actions::validate(&action)?;
    crate::engine::visuals::validate_button_visual(&visual)?;
    let mut cfg = state.config.lock().unwrap();
    if !cfg.obs.enabled {
        return Err("obs_disabled".into());
    }
    let p = cfg.active_paleta().ok_or("tab_not_found")?;
    if !p.obs {
        return Err("obs_separate_palettes".into());
    }
    if index == 0 || index > p.rows.saturating_mul(p.cols) {
        return Err("button_not_found".into());
    }
    // Validar el borrador completo antes de modificar el estado compartido.
    let mut next = cfg.clone();
    rules::apply_button_shortcut(
        &mut next,
        index,
        &shortcut,
        replace_shortcut.unwrap_or(false),
    )?;
    midi_rules::apply_button(&mut next, index, &midi, replace_shortcut.unwrap_or(false))?;
    let p = next.active_paleta_mut().ok_or("tab_not_found")?;
    let mut b = new_button(&p.id, index, &label, &color_bg, &color_text);
    b.type_field = "obs".into();
    b.obs = Some(action);
    b.shortcut = shortcut;
    b.midi = midi;
    b.visual = visual;
    p.botones.retain(|b| b.index != index);
    p.botones.push(b);
    config_io::save_config(&next)?;
    *cfg = next;
    drop(cfg);
    keyboard::sync(&app)
}
/// Única ejecución OBS desde clic, teclado y MIDI; exige la paleta visible.
pub fn execute(state: &AppState, id: &str) -> Result<(), String> {
    if cfg!(target_os = "android") { return Err("platform_unavailable".into()); }
    let cfg = state.config.lock().unwrap();
    if !cfg.obs.enabled {
        return Err("obs_disabled".into());
    }
    let p = cfg.active_paleta().ok_or("tab_not_found")?;
    if !p.obs {
        return Err("obs_separate_palettes".into());
    }
    let b = p
        .botones
        .iter()
        .find(|b| b.id == id)
        .ok_or("button_not_found")?;
    let action = b.obs.clone().ok_or("obs_invalid_action")?;
    drop(cfg);
    state.obs.execute(id.into(), action)
}
