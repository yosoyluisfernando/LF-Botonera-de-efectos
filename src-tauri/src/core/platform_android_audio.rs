//! CPAL/Oboe comparte el contexto JNI de Android con el proceso de Tauri.
use std::sync::OnceLock;
use tauri::tao::platform::android::prelude::{jni, main_android_context, GlobalRef, JObject};

static CONTEXT: OnceLock<Result<GlobalRef, String>> = OnceLock::new();

pub(super) fn initialize() -> Result<(), String> {
    CONTEXT
        .get_or_init(application_context)
        .as_ref()
        .map(|_| ())
        .map_err(Clone::clone)
}

fn application_context() -> Result<GlobalRef, String> {
    let context = main_android_context().ok_or("Tauri todavía no tiene una actividad Android")?;
    // Tao mantiene el puntero a la VM y una referencia global a la actividad durante el setup.
    let vm = unsafe { jni::JavaVM::from_raw(context.java_vm.cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(context.context_jobject.cast()) };
    let application = env
        .call_method(
            &activity,
            "getApplicationContext",
            "()Landroid/content/Context;",
            &[],
        )
        .and_then(|value| value.l())
        .map_err(|e| e.to_string())?;
    if application.is_null() {
        return Err("Android no devolvió el contexto de la aplicación".into());
    }
    let global = env.new_global_ref(application).map_err(|e| e.to_string())?;
    // OnceLock garantiza una sola inicialización antes de crear los hilos de audio.
    // La referencia a Application permanece válida aunque se recree la Activity.
    unsafe {
        ndk_context::initialize_android_context(context.java_vm, global.as_obj().as_raw().cast())
    };
    Ok(global)
}
