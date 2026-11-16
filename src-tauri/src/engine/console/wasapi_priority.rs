//! Prioridad multimedia del hilo que entrega los bloques de salida a WASAPI.
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{
    AvRevertMmThreadCharacteristics, AvSetMmThreadCharacteristicsW,
};

thread_local! {
    // Varios buses pueden compartir el mismo hilo de salida.
    static REGISTRATION: Option<AudioRegistration> = AudioRegistration::new();
}

/// Devuelve true cuando este hilo ya intentó registrarse en MMCSS.
pub(super) fn register_on_output_thread() -> bool {
    // Rodio puede pedir muestras al construir un remuestreador. CPAL 0.15.3
    // asigna este nombre al hilo de salida, donde debe registrarse MMCSS.
    if std::thread::current().name() != Some("cpal_wasapi_out") {
        return false;
    }
    REGISTRATION.with(|_| ());
    true
}

struct AudioRegistration(HANDLE);

impl AudioRegistration {
    fn new() -> Option<Self> {
        let mut index = 0;
        // El nombre UTF-16 es estático y Windows solo escribe en index durante la llamada.
        let handle = unsafe { AvSetMmThreadCharacteristicsW(windows_sys::w!("Audio"), &mut index) };
        (!handle.is_null()).then(|| Self(handle))
    }
}

impl Drop for AudioRegistration {
    fn drop(&mut self) {
        // El TLS libera el registro en el mismo hilo que lo solicitó.
        unsafe { AvRevertMmThreadCharacteristics(self.0) };
    }
}
