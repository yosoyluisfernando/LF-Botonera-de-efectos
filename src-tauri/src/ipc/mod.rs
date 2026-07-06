pub mod cmd_audio;
pub mod cmd_button_flags;
pub mod cmd_button_playback;
pub mod cmd_button_types;
pub mod cmd_button_update;
pub mod cmd_config;
pub mod cmd_export;
pub mod cmd_grid;
pub mod cmd_history;
pub mod cmd_keys;
pub mod cmd_local_shortcuts;
pub mod cmd_locutions;
pub mod cmd_master_volume;
pub mod cmd_meta;
pub mod cmd_norm;
pub mod cmd_paletas;
pub mod cmd_playback;
pub mod cmd_playback_progress;
pub mod cmd_preload;
pub mod cmd_profiles;
pub mod cmd_startup_prompts;
pub mod cmd_tracks;
pub mod cmd_updates;

#[macro_use]
pub mod register;

pub use crate::core::AppState;
