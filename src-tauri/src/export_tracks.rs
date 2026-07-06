/// Módulo: export_tracks.rs
/// Propósito: empaquetar y restaurar los metadatos del editor de pista (cue, dB,
/// normalización) en los .bdelf/.bdeplf para que las configuraciones viajen con
/// la exportación y no se pierdan. Se guardan en un campo OPCIONAL `bdelf_tracks`
/// (ruta→metadatos) que el LF Automatizador ignora → compatibilidad bidireccional
/// (Regla 5). La lógica vive en Rust; la UI no interviene.
use crate::audio_analysis::file_stamp;
use crate::track_store::TrackStore;
use crate::model::{ButtonData, PaletaData};
use serde_json::{Map, Value};

fn is_audio(b: &ButtonData) -> bool {
    b.type_field == "audio" && !b.path.is_empty()
}

/// Rutas de audio de una pestaña.
pub fn paleta_paths(p: &PaletaData) -> Vec<String> {
    p.botones.iter().filter(|b| is_audio(b)).map(|b| b.path.clone()).collect()
}

/// Inserta `bdelf_tracks` en el JSON exportado (solo de las rutas con datos).
pub fn inject(value: &mut Value, store: &TrackStore, paths: &[String]) {
    let mut map = Map::new();
    for path in paths {
        if let Ok(Some(meta)) = store.get(path) {
            if let Ok(v) = serde_json::to_value(&meta) {
                map.insert(path.clone(), v);
            }
        }
    }
    if !map.is_empty() {
        if let Some(obj) = value.as_object_mut() {
            obj.insert("bdelf_tracks".to_string(), Value::Object(map));
        }
    }
}

/// Restaura a tracks.db los metadatos traídos en `bdelf_tracks`. Re-sella el
/// mtime/size al archivo actual para que el cue aplique en esta máquina; si el
/// archivo no existe, conserva los datos igualmente (no se pierden).
pub fn restore(store: &TrackStore, tracks: &Value) {
    let Some(obj) = tracks.as_object() else {
        return;
    };
    for (path, mv) in obj {
        let Ok(meta) = serde_json::from_value::<crate::model::track::TrackMeta>(mv.clone()) else {
            continue;
        };
        let mut base = meta.clone();
        let (mtime, size) = file_stamp(path);
        if size > 0 {
            base.mtime = mtime;
            base.size = size;
        }
        // upsert crea/refresca la fila; los setters fijan el cue/dB importados
        // (upsert preserva el cue existente, así que hay que escribirlo aparte).
        let _ = store.upsert(&base);
        let _ = store.set_cue(path, meta.cue_start_s, meta.cue_end_s);
        let _ = store.set_gain(path, meta.gain_db);
        let _ = store.set_normalization(path, meta.norm_enabled);
    }
}
