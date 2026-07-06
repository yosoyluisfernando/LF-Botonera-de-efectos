/// Modulo: audio_device_list.rs
/// Proposito: listar salidas de audio disponibles.
use rodio::cpal::traits::{DeviceTrait, HostTrait};

pub fn available_devices() -> Vec<String> {
    let host = rodio::cpal::default_host();
    let mut devices = vec!["default".to_string()];
    if let Ok(devs) = host.output_devices() {
        for device in devs {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
    }
    devices
}
