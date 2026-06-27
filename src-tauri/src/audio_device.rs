/// Modulo: audio_device.rs
/// Proposito: mantener el dispositivo de salida activo del hilo de audio.
use crate::master_bus::{ButtonStateMap, MasterBus};
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{OutputStream, OutputStreamHandle};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};

pub struct AudioDeviceRuntime {
    bus: Option<MasterBus>,
    stream_data: Option<(OutputStream, OutputStreamHandle)>,
    current_device: String,
}

impl AudioDeviceRuntime {
    pub fn new() -> Self {
        Self {
            bus: None,
            stream_data: None,
            current_device: String::new(),
        }
    }

    pub fn bus(&self) -> Option<&MasterBus> {
        self.bus.as_ref()
    }

    /// Suelta el bus y el stream (p.ej. al desactivar la pre-escucha).
    pub fn clear(&mut self) {
        self.bus = None;
        self.stream_data = None;
        self.current_device = String::new();
    }

    /// Crea el bus en el dispositivo dado. `clear_states` vacía las fuentes
    /// activas (solo el bus principal lo usa; la pre nunca toca el estado).
    pub fn set_device(
        &mut self,
        states: &Arc<Mutex<ButtonStateMap>>,
        master_l: &Arc<AtomicU32>,
        master_r: &Arc<AtomicU32>,
        master_volume: &Arc<AtomicU32>,
        device_name: String,
        clear_states: bool,
    ) {
        if device_name == self.current_device && self.bus.is_some() {
            return;
        }
        let Some(device) = find_device(&device_name) else {
            return;
        };
        if let Ok((stream, handle)) = OutputStream::try_from_device(&device) {
            if clear_states {
                states.lock().unwrap().clear();
            }
            self.bus = MasterBus::new(
                &handle,
                Arc::clone(master_l),
                Arc::clone(master_r),
                Arc::clone(master_volume),
            );
            self.stream_data = Some((stream, handle));
            self.current_device = device_name;
        }
    }
}

fn find_device(device_name: &str) -> Option<rodio::cpal::Device> {
    let host = rodio::cpal::default_host();
    if device_name == "default" {
        return host.default_output_device();
    }
    host.output_devices().ok().and_then(|mut devices| {
        devices.find(|device| device.name().unwrap_or_default() == device_name)
    })
}
