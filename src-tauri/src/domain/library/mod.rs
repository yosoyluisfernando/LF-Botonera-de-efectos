//! Reglas puras de la Biblioteca. Sin SQLite, sistema de archivos ni UI.
pub mod metadata_fields;
pub mod path_rewrite;
pub mod protected_tracks;
pub mod root_plan;

pub use root_plan::{
    owner_for_path, plan_root_add, LibraryCollection, RootAddKind, RootAddPlan, RootSpec,
};
