/// Modulo: engine/console/endpoint.rs
/// Proposito: OutputEndpoint = una tarjeta fisica abierta EXACTAMENTE una vez.
///
/// rodio ya trae el mixer de salida dentro de OutputStream (`play_raw` anade a
/// el), asi que un endpoint no construye nada: mantiene el stream vivo y reparte
/// su handle. Varios buses enchufados a la misma tarjeta se suman EN EL CONECTOR
/// sin tocarse entre ellos, que es justo lo que separa "senal" de "conector".
///
/// OutputStream NO es Send (lleva un cpal::Stream dentro): por eso el registro
/// entero vive en el hilo guardian de la consola y no se mueve de ahi. Lo que
/// viaja a los demas motores es OutputStreamHandle, que si es Clone + Send + Sync.
use super::device::find_device;
use rodio::{OutputStream, OutputStreamHandle};
use std::collections::HashMap;

pub struct OutputEndpoint {
    /// Mantiene la tarjeta abierta. Si se suelta, el Weak del handle deja de
    /// resolver y `play_raw` empieza a fallar: por eso se guarda aunque no se lea.
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl OutputEndpoint {
    fn open(device_name: &str) -> Option<Self> {
        let device = find_device(device_name)?;
        let (stream, handle) = OutputStream::try_from_device(&device).ok()?;
        Some(Self {
            _stream: stream,
            handle,
        })
    }

    pub fn handle(&self) -> &OutputStreamHandle {
        &self.handle
    }
}

/// Las tarjetas abiertas, por nombre.
#[derive(Default)]
pub struct EndpointRegistry {
    open: HashMap<String, OutputEndpoint>,
}

impl EndpointRegistry {
    /// Devuelve el endpoint de esa tarjeta, abriendola si aun no lo estaba.
    /// Pedir dos veces la misma tarjeta devuelve LA MISMA: ese es el registro.
    pub fn ensure(&mut self, device_name: &str) -> Option<&OutputEndpoint> {
        if !self.open.contains_key(device_name) {
            self.open
                .insert(device_name.to_string(), OutputEndpoint::open(device_name)?);
        }
        self.open.get(device_name)
    }

    /// Cierra las tarjetas que ya no usa ningun bus. Se llama tras cada cambio de
    /// ruteo: una tarjeta abierta que nadie escucha puede dejar sin ella a otro
    /// programa (WASAPI en exclusivo) y no aporta nada.
    pub fn retain_only(&mut self, in_use: &[String]) {
        self.open.retain(|name, _| in_use.contains(name));
    }
}
