pub mod actions;
#[cfg(not(target_os = "android"))]
pub mod keyboard;
#[cfg(target_os = "android")]
#[path = "keyboard_mobile.rs"]
pub mod keyboard;
pub mod midi;
#[cfg(target_os = "android")]
pub mod midi_android;
#[cfg(not(target_os = "android"))]
pub mod midi_backend;
#[cfg(any(target_os = "android", test))]
mod midi_stream;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod midi_ble;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod midi_ble_connection;
#[cfg(any(target_os = "windows", target_os = "linux", test))]
mod midi_ble_packet;
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
