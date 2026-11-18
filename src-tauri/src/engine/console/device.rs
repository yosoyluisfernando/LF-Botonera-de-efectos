//! Salidas: nombres heredados, alias del sistema e identidades persistentes.
pub use super::device_catalog::{OutputDevice, descriptor, label as display_name, list as output_devices, refresh as refresh_outputs};
pub fn available_devices() -> Vec<String> {
    if cfg!(target_os = "android") { return vec!["default".into()]; }
    let mut devices = vec!["default".into()];
    devices.extend(refresh_outputs().into_iter().map(|d| d.name));
    devices
}
pub fn device_available(name: &str) -> bool { find_device(name).is_some() }
pub fn find_device(name: &str) -> Option<rodio::cpal::Device> {
    if cfg!(target_os = "android") {
        use rodio::cpal::traits::HostTrait;
        return rodio::cpal::default_host().default_output_device();
    }
    super::device_catalog::find(name)
}
pub fn canonical_name(name: &str) -> String {
    if cfg!(target_os = "android") { return "default".into(); }
    super::device_catalog::resolve(name)
}
