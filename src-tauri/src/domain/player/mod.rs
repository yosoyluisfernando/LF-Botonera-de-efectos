//! Reglas puras del reproductor auxiliar (modo reproductor). Sin audio ni I/O.
pub mod advance;
pub mod queue_remove;

pub use advance::{next_index, PlayerMode};
