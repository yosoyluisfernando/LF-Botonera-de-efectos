/// Modulo: engine/console/device.rs
/// Proposito: localizar tarjetas de sonido por nombre y listar las disponibles.
/// Vive en la consola porque es ella quien abre las salidas fisicas: los demas
/// motores le piden buses o endpoints, no tarjetas.
use rodio::cpal::traits::{DeviceTrait, HostTrait};

/// Lista las salidas disponibles. "default" siempre va primero: no es el nombre
/// de ninguna tarjeta, es "la que diga el sistema".
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

pub fn device_available(device_name: &str) -> bool {
    find_device(device_name).is_some()
}

pub fn find_device(device_name: &str) -> Option<rodio::cpal::Device> {
    let host = rodio::cpal::default_host();
    if device_name == "default" {
        return host.default_output_device();
    }
    host.output_devices().ok().and_then(|mut devices| {
        devices.find(|device| device.name().unwrap_or_default() == device_name)
    })
}
