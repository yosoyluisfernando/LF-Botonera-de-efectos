use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::plugin::{Builder, PluginHandle, TauriPlugin};
use tauri::{AppHandle, Emitter, Manager, Wry};

struct NativeApp(PluginHandle<Wry>);

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new("android-app")
        .setup(|app, api| {
            let native = api.register_android_plugin("lf.botonera.app", "AppPlugin")?;
            app.manage(NativeApp(native));
            Ok(())
        })
        .build()
}

pub async fn attach_back(app: &AppHandle) -> Result<(), String> {
    let receiver = app.clone();
    let channel = Channel::<Value>::new(move |_| {
        let _ = receiver.emit("android-back", ());
        Ok(())
    });
    app.state::<NativeApp>()
        .0
        .run_mobile_plugin_async::<()>("attachBack", json!({ "channel": channel }))
        .await
        .map_err(|error| error.to_string())
}

pub async fn installer_source(app: &AppHandle) -> Result<String, String> {
    app.state::<NativeApp>()
        .0
        .run_mobile_plugin_async::<String>("getInstallerSource", ())
        .await
        .map_err(|error| error.to_string())
}

pub async fn exit(app: &AppHandle) -> Result<(), String> {
    app.state::<NativeApp>()
        .0
        .run_mobile_plugin_async::<()>("exitApp", ())
        .await
        .map_err(|error| error.to_string())
}
