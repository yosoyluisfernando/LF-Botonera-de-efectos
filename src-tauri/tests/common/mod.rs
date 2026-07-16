//! Lo compartido por las pruebas contra tarjetas reales: montar tonos, leer
//! niveles y encontrar las salidas del equipo.
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tauri_app_lib::engine::console::device::available_devices;

use rodio::buffer::SamplesBuffer;

/// Un momento para que el medidor tenga muestras que medir: mide por ventanas de
/// ~21 ms y la tarjeta pide el audio a su ritmo, no al nuestro.
pub fn respirar() {
    sleep(Duration::from_millis(250));
}

pub fn nivel(atomico: &Arc<AtomicU32>) -> f32 {
    f32::from_bits(atomico.load(Ordering::Relaxed))
}

/// Un tono continuo de un minuto: no se acaba a mitad de la prueba.
pub fn tono(nivel: f32) -> SamplesBuffer<f32> {
    SamplesBuffer::new(2, 48_000, vec![nivel; 48_000 * 2 * 60])
}

/// Las dos primeras tarjetas de verdad (saltando "default", que no es una tarjeta
/// sino "la que diga el sistema"). None si el equipo no tiene dos.
pub fn dos_tarjetas() -> Option<(String, String)> {
    let devs: Vec<String> = available_devices().into_iter().skip(1).collect();
    match devs.as_slice() {
        [a, b, ..] => Some((a.clone(), b.clone())),
        _ => None,
    }
}
