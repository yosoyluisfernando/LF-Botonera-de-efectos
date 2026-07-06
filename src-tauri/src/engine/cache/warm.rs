/// Módulo: preload_warm.rs
/// Propósito: estrategias de llenado PROACTIVO de la caché (FullProfile y
/// VisibleTabs). Decide QUÉ encolar según la config y la pestaña/perfil activos;
/// el `Preloader` hace el trabajo en segundo plano. OnPlay se gestiona al
/// disparar (cmd_button_playback). No bloquea: solo recolecta rutas y encola.
/// IMPORTANTE: no llamar con el lock de config tomado (volvería a bloquearlo).
use crate::model::ButtonData;
use crate::model::preload::PreloadStrategy;
use crate::AppState;

/// Encola según la estrategia activa. Para perfil completo: todos los audios
/// cortos del perfil; para pestañas visibles: los de la pestaña activa.
pub fn warm_for_strategy(state: &AppState) {
    enqueue_all(state, collect_paths(state));
}

/// Solo para VisibleTabs: encola la pestaña que se acaba de abrir. En
/// FullProfile ya está todo y en OnPlay no aplica → no hace nada.
pub fn warm_visible_tab(state: &AppState) {
    if state.config.lock().unwrap().preload.strategy != PreloadStrategy::VisibleTabs {
        return;
    }
    enqueue_all(state, collect_paths(state));
}

/// Recalentado al arrancar (estrategia OnPlay): encola los archivos cortos
/// reproducidos dentro de la ventana TTL (last_played reciente). Solo lee la DB.
pub fn warm_onplay_recent(state: &AppState) {
    let (enabled, on_play, hours, max) = {
        let p = &state.config.lock().unwrap().preload;
        (
            p.enabled,
            p.strategy == PreloadStrategy::OnPlay,
            p.evict_after_hours as i64,
            p.max_duration_s as f64,
        )
    };
    if !enabled || !on_play {
        return;
    }
    let since = chrono::Utc::now().timestamp() - hours * 3600;
    let paths = state
        .tracks
        .lock()
        .unwrap()
        .recent_paths(since, max)
        .unwrap_or_default();
    enqueue_all(state, paths);
}

fn collect_paths(state: &AppState) -> Vec<String> {
    let cfg = state.config.lock().unwrap();
    if !cfg.preload.enabled {
        return Vec::new();
    }
    let Some(profile) = cfg.profiles.iter().find(|p| p.id == cfg.active_profile_id) else {
        return Vec::new();
    };
    let max = cfg.preload.max_duration_s as f64;
    let mut out = Vec::new();
    for paleta in &profile.paletas {
        let include = match cfg.preload.strategy {
            PreloadStrategy::FullProfile => true,
            PreloadStrategy::VisibleTabs => paleta.id == profile.active_paleta_id,
            PreloadStrategy::OnPlay => false,
        };
        if include {
            for btn in &paleta.botones {
                if qualifies(btn, max) {
                    out.push(btn.path.clone());
                }
            }
        }
    }
    out
}

fn qualifies(btn: &ButtonData, max: f64) -> bool {
    btn.type_field == "audio" && !btn.path.is_empty() && btn.duration > 0.0 && btn.duration < max
}

fn enqueue_all(state: &AppState, paths: Vec<String>) {
    if paths.is_empty() {
        return;
    }
    let engine = state.audio.lock().unwrap();
    for p in paths {
        engine.enqueue_preload(p);
    }
}
