pub mod core;
pub mod domain;
pub mod engine;
#[macro_use]
pub mod ipc;
pub mod model;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;
    let builder = tauri::Builder::default();
    #[cfg(not(target_os = "android"))]
    let builder = builder
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .plugin(engine::input::keyboard::plugin())
        .plugin(tauri_plugin_drag::init());
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            core::platform::initialize(app.handle())?;
            app.manage(core::AppState::new());
            core::setup::on_setup(app)
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(lf_invoke_handlers!())
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicacion Tauri");
}
