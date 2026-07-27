//! Reglas puras de la Biblioteca. Sin SQLite, sistema de archivos ni UI.
pub mod root_plan;

pub use root_plan::{
    owner_for_path, plan_root_add, LibraryCollection, RootAddKind, RootAddPlan, RootSpec,
};
