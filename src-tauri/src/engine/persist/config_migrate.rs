//! Modulo: engine/persist/config_migrate.rs
//! Proposito: migraciones que se aplican al CARGAR la configuracion, para que un
//! archivo guardado por una version anterior siga siendo valido sin que el
//! usuario tenga que tocar nada. Cada una explica de que version viene y por que.
use crate::engine::audio::formats::probe_duration_secs;
use crate::model::{AppConfig, ButtonData};

/// Migracion: las listas del LFA importadas antes guardaron su MARCADOR
/// (`time_locution`…) como si fuera una carpeta, y asi la locucion no sonaba
/// nunca. Se vacia para que use la carpeta de Ajustes, como hace el LFA con las
/// suyas. Solo toca lo que es exactamente un marcador: una carpeta real se queda.
pub(crate) fn clear_locution_markers(cfg: &mut AppConfig) {
    for track in cfg.player.tracks.iter_mut() {
        let is_locution = matches!(track.type_field.as_str(), "time" | "temperature" | "humidity");
        let is_marker = matches!(
            track.folder.trim(),
            "time_locution" | "temperature_locution" | "humidity_locution"
        );
        if is_locution && is_marker {
            track.folder.clear();
        }
    }
}

/// Garantiza que los ids de botón sean únicos entre pestañas
/// (formato "{paleta_id}_btn_{index}"). Migra configs con el formato
/// antiguo "btn_{index}", que colisionaba entre pestañas.
pub(crate) fn normalize_button_ids(cfg: &mut AppConfig) {
    for profile in cfg.profiles.iter_mut() {
        for paleta in profile.paletas.iter_mut() {
            let pid = paleta.id.clone();
            for b in paleta.botones.iter_mut() {
                let expected = format!("{}_btn_{}", pid, b.index);
                if b.id != expected {
                    b.id = expected;
                }
            }
        }
    }
}

pub(crate) fn normalize_playback_modes(cfg: &mut AppConfig) {
    for profile in cfg.profiles.iter_mut() {
        if profile.audio.playback_mode == "stop_others" {
            profile.audio.playback_mode = "normal".to_string();
            profile.audio.solo_mode = true;
        }
    }
    // El reproductor tuvo un modo `manual` que no avanzaba solo. Se quito porque
    // duplicaba el boton "detener al finalizar" y ademas limitaba: para elegir la
    // siguiente forzaba el orden normal, asi que no podia combinarse con
    // aleatorio. Cae a `normal`, que es lo que hacia para elegir. Lo de "no
    // avanzar solo" lo da ahora ese boton, que no se persiste a proposito.
    if cfg.player.playback_mode == "manual" {
        cfg.player.playback_mode = "normal".to_string();
    }
}

/// Migracion: recupera la duracion de los archivos que se guardaron sin ella.
/// `probe_duration_secs` pedia las etiquetas de paso, y un MP3 con el titulo mal
/// codificado (frecuente en rippeos viejos con acentos) tumbaba la lectura
/// entera: se perdia la duracion, que no tiene nada que ver con el texto. Ya
/// esta arreglado, pero las configuraciones guardadas conservan el `-1`.
///
/// **Cubre TODO lo que tiene duracion**, no solo la cola del reproductor: los
/// botones arrastran el fallo desde mucho antes, y `get_grid_state` solo
/// reintenta cuando vale `0`, no cuando vale `-1`. Sin esto, un boton atascado
/// en `-1` no se recuperaria nunca.
pub(crate) fn recover_missing_durations(cfg: &mut AppConfig) {
    for track in cfg.player.tracks.iter_mut() {
        retry_duration(track);
    }
    for btn in cfg.fixed_panel.global_buttons.iter_mut() {
        retry_duration(btn);
    }
    for profile in cfg.profiles.iter_mut() {
        for btn in profile.fixed_buttons.iter_mut() {
            retry_duration(btn);
        }
        for paleta in profile.paletas.iter_mut() {
            for btn in paleta.botones.iter_mut() {
                retry_duration(btn);
            }
        }
    }
}

/// Solo reintenta lo que puede fallar por esto: un audio con ruta y sin duracion.
/// Cuesta ~40 ms por archivo, asi que no se toca lo que ya la tiene.
fn retry_duration(btn: &mut ButtonData) {
    if btn.type_field != "audio" || btn.duration > 0.0 || btn.path.is_empty() {
        return;
    }
    let secs = probe_duration_secs(&btn.path);
    if secs > 0.0 {
        btn.duration = secs;
        btn.duration_str = format!("{secs:.1}s");
    }
}

#[cfg(test)]
#[path = "config_migrate_tests.rs"]
mod config_migrate_tests;
