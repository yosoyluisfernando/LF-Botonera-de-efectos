//! Modulo: domain/console/tab_lanes.rs
//! Proposito: decidir por que carril (`BusId::Salida`) sale cada pestaña que
//! eligio una tarjeta propia. Pura: recibe nombres de tarjetas, no las abre.
//!
//! Adaptado de Danilo Wenner, commit 7c1f1ae (GPL-3.0-or-later).
//! Reglas:
//! - Pestaña sin salida, o con la misma que la principal → bus Efectos (como
//!   siempre: suma en el programa y obedece al master).
//! - Varias pestañas con la misma tarjeta comparten carril.
//! - Las asignaciones son ESTABLES: cambiar la salida de una pestaña no mueve a
//!   las demas de carril (moverlas cortaria lo que suena por ellas).
//! - Una tarjeta ausente conserva su carril y permanece en silencio, sin desvio.
use super::routing::{BusId, Routing, MAX_SALIDAS};
use serde::Serialize;
use std::collections::HashMap;

/// Lo minimo de una pestaña que hace falta para rutearla.
pub struct TabRoute<'a> {
    pub paleta_id: &'a str,
    pub nombre: &'a str,
    pub audio_out: &'a str,
}

/// Una pestaña que no puede salir por la tarjeta que pidio.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutputIssue {
    pub paleta_id: String,
    pub nombre: String,
    pub device: String,
    /// "missing" = la tarjeta no esta conectada; "too_many" = no quedan carriles.
    pub reason: &'static str,
}

#[derive(Debug, Default)]
pub struct LanePlan {
    /// tarjeta → carril. Se guarda para que el proximo plan sea estable.
    pub lanes: HashMap<String, u8>,
    /// pestaña → carril, solo para las que tienen tarjeta propia.
    pub paleta_bus: HashMap<String, BusId>,
    /// El ruteo de TODOS los carriles (los libres, sin tarjeta).
    pub routings: Vec<(BusId, Routing)>,
    pub issues: Vec<OutputIssue>,
}

/// Si una salida pedida es, en realidad, la principal.
pub fn is_main_output(audio_out: &str, main: &str) -> bool {
    let out = audio_out.trim();
    let main = if main.trim().is_empty() {
        "default"
    } else {
        main.trim()
    };
    out.is_empty() || out == main
}

pub fn plan(
    tabs: &[TabRoute],
    main: &str,
    available: &[String],
    previous: &HashMap<String, u8>,
) -> LanePlan {
    let mut out = LanePlan::default();
    let mut wanted: Vec<&str> = Vec::new();
    for tab in tabs {
        let device = tab.audio_out.trim();
        if is_main_output(device, main) {
            continue;
        }
        if !available.iter().any(|d| d == device) {
            out.issues.push(issue(tab, "missing"));
        }
        if !wanted.contains(&device) {
            wanted.push(device);
        }
    }
    // Primero se conservan los carriles que ya tenian dueño...
    for device in &wanted {
        if let Some(lane) = previous.get(*device) {
            out.lanes.insert(device.to_string(), *lane);
        }
    }
    // ...y luego las tarjetas nuevas ocupan el carril libre mas bajo.
    for device in &wanted {
        if out.lanes.contains_key(*device) {
            continue;
        }
        let free = (0..MAX_SALIDAS).find(|lane| !out.lanes.values().any(|l| l == lane));
        if let Some(lane) = free {
            out.lanes.insert(device.to_string(), lane);
        }
    }
    for tab in tabs {
        let device = tab.audio_out.trim();
        if is_main_output(device, main) {
            continue;
        }
        match out.lanes.get(device) {
            Some(lane) => {
                out.paleta_bus
                    .insert(tab.paleta_id.to_string(), BusId::Salida(*lane));
            }
            None => {
                out.paleta_bus
                    .insert(tab.paleta_id.to_string(), BusId::Salida(MAX_SALIDAS));
                out.issues.push(issue(tab, "too_many"));
            }
        }
    }
    out.routings = BusId::salidas()
        .map(|bus| {
            let device = out
                .lanes
                .iter()
                .find(|(_, lane)| BusId::Salida(**lane) == bus)
                .map(|(device, _)| device.clone())
                .unwrap_or_default();
            (bus, Routing::Device(device))
        })
        .collect();
    out
}

/// La pestaña a la que pertenece un boton de la rejilla, por su id
/// (`{paleta_id}_btn_{index}`). None para ids especiales o del panel.
pub fn paleta_of_button(button_id: &str) -> Option<&str> {
    button_id.rsplit_once("_btn_").map(|(paleta, _)| paleta)
}

fn issue(tab: &TabRoute, reason: &'static str) -> OutputIssue {
    OutputIssue {
        paleta_id: tab.paleta_id.to_string(),
        nombre: tab.nombre.to_string(),
        device: tab.audio_out.trim().to_string(),
        reason,
    }
}

#[cfg(test)]
#[path = "tab_lanes_tests.rs"]
mod tab_lanes_tests;
