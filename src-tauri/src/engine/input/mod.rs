pub mod actions;
#[cfg(not(target_os = "android"))]
pub mod keyboard;
#[cfg(target_os = "android")]
#[path = "keyboard_mobile.rs"]
pub mod keyboard;
pub mod midi;
pub mod midi_backend;
pub mod midi_binding;
#[cfg(test)]
mod midi_capture_tests;
pub mod midi_dispatch;
pub mod midi_message;
pub mod midi_ports;
pub mod midi_rules;
pub mod profile_shortcuts;
#[cfg(test)]
mod profile_shortcuts_tests;
pub mod rules;
pub mod tab_reorder;

#[cfg(test)]
mod mic_shortcuts_tests;
#[cfg(test)]
mod profile_shortcuts_conflict_tests;
