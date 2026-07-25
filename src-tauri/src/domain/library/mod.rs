//! Reglas puras de la Biblioteca. Sin SQLite, sistema de archivos ni UI.
pub mod root_plan;

pub use root_plan::{plan_root_add, LibraryCollection, RootAddKind, RootAddPlan, RootSpec};
