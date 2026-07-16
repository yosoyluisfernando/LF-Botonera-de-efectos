//! Modulo: domain/export/lfa_format/playlist.rs
//! Proposito: adaptador del formato de listas del LF Automatizador (.LFPlay):
//! un JSON con un array de filas `{ruta, titulo, duracion, type, target}`.
//! Compatibilidad bidireccional: lo que guardamos abre en el Automatizador y lo
//! suyo abre aqui. Las filas de comando del LFA (notas, saltos, eventos) no
//! aplican a la botonera y se ignoran al importar.
use crate::domain::button::defaults::new_button;
use crate::model::ButtonData;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct LfaPlaylistRow {
    #[serde(default)]
    pub ruta: String,
    #[serde(default)]
    pub titulo: String,
    /// El Automatizador lee `duracion ?? duration` con `parseInt`, asi que
    /// aceptamos ambos nombres y ambos tipos (numero o cadena). Ver `flexible_secs`.
    #[serde(default, alias = "duration", deserialize_with = "flexible_secs")]
    pub duracion: f64,
    #[serde(default = "default_row_type", rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub target: Option<String>,
}

fn default_row_type() -> String {
    "normal".to_string()
}

/// El Automatizador NO guarda carpeta en las locuciones: escribe un marcador en
/// `ruta` y resuelve los archivos con su configuracion global. Si lo tomaramos
/// por una carpeta buscariamos un directorio llamado "time_locution" y la
/// locucion no sonaria nunca. Carpeta vacia = la de Ajustes, igual que el LFA.
fn marker_for(kind: &str) -> Option<&'static str> {
    match kind {
        "time" => Some("time_locution"),
        "temperature" => Some("temperature_locution"),
        "humidity" => Some("humidity_locution"),
        _ => None,
    }
}

fn is_marker(ruta: &str) -> bool {
    matches!(
        ruta.trim(),
        "time_locution" | "temperature_locution" | "humidity_locution"
    )
}

/// Lee la duracion venga como numero (`172`) o como cadena (`"31"`): el
/// Automatizador escribe ambas segun la version que guardo la lista, y el suyo lo
/// resuelve con `parseInt`. Una duracion ilegible vale 0 en vez de tumbar el
/// archivo entero: mas vale una cancion sin duracion que perder la lista.
fn flexible_secs<'de, D: Deserializer<'de>>(de: D) -> Result<f64, D::Error> {
    Ok(match Value::deserialize(de)? {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    })
}

/// Pista de la botonera → fila del Automatizador.
pub fn to_lfa_row(btn: &ButtonData) -> LfaPlaylistRow {
    let (type_field, ruta) = match btn.type_field.as_str() {
        "random_folder" => ("random", btn.folder.clone()),
        // Sin carpeta propia va el marcador del LFA: una `ruta` vacia le dejaria
        // una fila que no sabria resolver.
        "time" | "temperature" | "humidity" => {
            let ruta = if btn.folder.trim().is_empty() {
                marker_for(&btn.type_field).unwrap_or_default().to_string()
            } else {
                btn.folder.clone()
            };
            (btn.type_field.as_str(), ruta)
        }
        _ => ("normal", btn.path.clone()),
    };
    LfaPlaylistRow {
        ruta,
        titulo: if btn.name.is_empty() { btn.label.clone() } else { btn.name.clone() },
        duracion: btn.duration.max(0.0),
        type_field: type_field.to_string(),
        target: None,
    }
}

/// Fila del Automatizador → pista de la botonera. `None` para lo que no
/// soportamos: notas, saltos y demas comandos de automatizacion, o audio sin ruta.
pub fn from_lfa_row(row: &LfaPlaylistRow, index: u32, bg: &str, text: &str) -> Option<ButtonData> {
    let kind = match row.type_field.as_str() {
        "normal" | "" => "audio",
        "random" => "random_folder",
        "time" | "temperature" | "humidity" => row.type_field.as_str(),
        _ => return None,
    };
    if kind == "audio" && row.ruta.trim().is_empty() {
        return None;
    }
    let title = if row.titulo.trim().is_empty() { stem(&row.ruta) } else { row.titulo.clone() };
    let mut btn = new_button("player", index, &title, bg, text);
    btn.type_field = kind.to_string();
    if kind == "audio" {
        btn.path = row.ruta.clone();
    } else if !is_marker(&row.ruta) {
        // Un marcador no es una carpeta: se deja vacia para que la locucion use
        // la carpeta configurada en Ajustes, que es lo que hace el LFA.
        btn.folder = row.ruta.clone();
    }
    btn.duration = row.duracion.max(0.0);
    btn.duration_str = if btn.duration > 0.0 {
        format!("{:.1}s", btn.duration)
    } else {
        String::new()
    };
    Some(btn)
}

fn stem(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_uppercase())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "playlist_tests.rs"]
mod playlist_tests;
