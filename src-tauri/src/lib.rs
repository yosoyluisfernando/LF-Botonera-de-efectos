pub mod core;
pub mod domain;
pub mod engine;
#[macro_use]
pub mod ipc;
pub mod model;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(error) = engine::backup::recover_pending_restore() {
        eprintln!("backup restore recovery unavailable: {error}");
    }
    if let Err(error) = engine::library::file_rename::recover_pending() {
        eprintln!("library rename recovery unavailable: {error}");
    }
    if let Err(error) = engine::library::tag_write_journal::recover() {
        eprintln!("metadata write recovery unavailable: {error}");
    }
    tauri::Builder::default()
        .manage(core::AppState::new())
        .setup(core::setup::on_setup)
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(lf_invoke_handlers!())
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicacion Tauri");
}
