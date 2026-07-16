//! Modulo: domain/console/mod.rs
//! Proposito: las reglas de la consola de audio, puras y sin hardware.
//! El motor (`engine/console/`) las obedece; aqui se deciden.
pub mod routing;

pub use routing::{device_of, effective, sanitize, BusId, Routing};
