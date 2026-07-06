/// Módulo: cmd_preload.rs
/// Propósito: IPC de la configuración de PRECARGA. La UI solo recoge opciones;
/// AQUÍ Rust valida (RAM permitida, rangos), convierte (días→horas), aplica
/// defaults, decide si mostrar el diálogo de primer arranque y persiste. La UI
/// no calcula ni decide nada (Regla 4).
use super::AppState;
use crate::config;
use crate::model::preload::{PreloadConfig, PreloadStrategy};
use serde::Serialize;

/// Vista para la UI: el TTL ya viene partido en {valor, unidad} calculado por
/// Rust, para que el frontend no haga aritmética de horas/días.
#[derive(Serialize)]
pub struct PreloadView {
    pub enabled: bool,
    pub ram_budget_mb: u32,
    pub max_duration_s: u32,
    pub strategy: String,
    pub evict_value: u32,
    pub evict_unit: String,
    pub prompted: bool,
}

impl From<&PreloadConfig> for PreloadView {
    fn from(c: &PreloadConfig) -> Self {
        let (evict_value, evict_unit) = if c.evict_after_hours >= 24 && c.evict_after_hours % 24 == 0
        {
            (c.evict_after_hours / 24, "days")
        } else {
            (c.evict_after_hours, "hours")
        };
        Self {
            enabled: c.enabled,
            ram_budget_mb: c.ram_budget_mb,
            max_duration_s: c.max_duration_s,
            strategy: c.strategy.as_str().to_string(),
            evict_value,
            evict_unit: evict_unit.to_string(),
            prompted: c.prompted,
        }
    }
}

#[tauri::command]
pub fn get_preload_config(state: tauri::State<AppState>) -> PreloadView {
    PreloadView::from(&state.config.lock().unwrap().preload)
}

/// Uso de la caché de precarga, para el indicador de Ajustes.
#[derive(Serialize)]
pub struct PreloadStats {
    pub used_mb: f64,
    pub count: usize,
    pub budget_mb: u32,
    pub enabled: bool,
}

#[tauri::command]
pub fn get_preload_stats(state: tauri::State<AppState>) -> PreloadStats {
    let (bytes, count) = state
        .audio
        .lock()
        .unwrap()
        .preload_cache_handle()
        .lock()
        .unwrap()
        .stats();
    let (budget_mb, enabled) = {
        let p = &state.config.lock().unwrap().preload;
        (p.ram_budget_mb, p.enabled)
    };
    PreloadStats {
        used_mb: bytes as f64 / (1024.0 * 1024.0),
        count,
        budget_mb,
        enabled,
    }
}

/// ¿Hay que mostrar el diálogo de primer arranque? Lo decide Rust.
#[tauri::command]
pub fn should_prompt_preload(state: tauri::State<AppState>) -> bool {
    !state.config.lock().unwrap().preload.prompted
}

/// Marca que ya se preguntó (botón "Ahora no" del diálogo).
#[tauri::command]
pub fn mark_preload_prompted(state: tauri::State<AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.preload.prompted = true;
    config::save_config(&cfg)
}

/// Guarda la configuración elegida por el usuario. Rust valida y convierte.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn set_preload_config(
    enabled: bool,
    ram_budget_mb: u32,
    max_duration_s: u32,
    strategy: String,
    evict_value: u32,
    evict_unit: String,
    state: tauri::State<AppState>,
) -> Result<PreloadView, String> {
    let (view, budget, active) = {
        let mut cfg = state.config.lock().unwrap();
        {
            let p = &mut cfg.preload;
            p.enabled = enabled;
            p.ram_budget_mb = snap_ram(ram_budget_mb);
            p.max_duration_s = max_duration_s.clamp(1, 60);
            p.strategy = PreloadStrategy::from_str(&strategy);
            p.evict_after_hours = to_hours(evict_value, &evict_unit);
            p.prompted = true;
        }
        config::save_config(&cfg)?;
        (
            PreloadView::from(&cfg.preload),
            cfg.preload.ram_budget_mb,
            cfg.preload.enabled,
        )
    };
    apply_runtime_preload(&state, budget, active);
    Ok(view)
}

fn apply_runtime_preload(state: &tauri::State<AppState>, budget: u32, enabled: bool) {
    let engine = state.audio.lock().unwrap();
    engine.set_preload_enabled(enabled);
    let cache = engine.preload_cache_handle();
    drop(engine);
    let mut cache = cache.lock().unwrap();
    cache.set_budget(budget);
    if !enabled {
        cache.clear();
        return;
    }
    drop(cache);
    crate::preload_warm::warm_for_strategy(state);
    crate::preload_warm::warm_onplay_recent(state);
}

/// Ajusta la RAM al valor permitido más cercano (32/64/128/256).
fn snap_ram(mb: u32) -> u32 {
    const ALLOWED: [u32; 4] = [32, 64, 128, 256];
    ALLOWED
        .into_iter()
        .min_by_key(|v| v.abs_diff(mb))
        .unwrap_or(128)
}

/// Convierte el TTL {valor, unidad} a horas y lo acota a un rango sensato.
fn to_hours(value: u32, unit: &str) -> u32 {
    let hours = if unit == "days" {
        value.saturating_mul(24)
    } else {
        value
    };
    hours.clamp(1, 24 * 365)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_ram_to_nearest_allowed() {
        assert_eq!(snap_ram(40), 32);
        assert_eq!(snap_ram(100), 128);
        assert_eq!(snap_ram(9999), 256);
    }

    #[test]
    fn days_convert_to_hours() {
        assert_eq!(to_hours(3, "days"), 72);
        assert_eq!(to_hours(5, "hours"), 5);
        assert_eq!(to_hours(0, "hours"), 1); // acotado a mínimo 1
    }

    #[test]
    fn view_splits_hours_into_days_when_exact() {
        let mut c = PreloadConfig::default();
        c.evict_after_hours = 72;
        let v = PreloadView::from(&c);
        assert_eq!((v.evict_value, v.evict_unit.as_str()), (3, "days"));
        c.evict_after_hours = 5;
        let v = PreloadView::from(&c);
        assert_eq!((v.evict_value, v.evict_unit.as_str()), (5, "hours"));
    }
}
