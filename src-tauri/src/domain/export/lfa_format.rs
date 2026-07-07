/// Modulo: lfa_format.rs
/// Proposito: fachada del formato .bdelf/.bdeplf compatible con LF Automatizador.
/// Las conversiones viven separadas para mantener cada archivo bajo 200 lineas.
mod paleta;
mod profile;
mod types;

pub use paleta::{from_lfa_paleta, to_lfa_paleta};
pub use profile::{from_lfa_profile, to_lfa_profile};
pub use types::{LfaButton, LfaConfig, LfaKeys, LfaPaleta, LfaProfile};
