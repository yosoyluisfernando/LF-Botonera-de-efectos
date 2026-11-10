use crate::{
    core::AppState,
    engine::remote::{protocol, RemoteStatus},
};

#[tauri::command]
pub fn remote_status(state: tauri::State<AppState>) -> RemoteStatus {
    state.remote.lock().unwrap().status()
}
#[tauri::command]
pub fn remote_start(
    app: tauri::AppHandle,
    ip: String,
    port: u16,
    state: tauri::State<AppState>,
) -> Result<RemoteStatus, String> {
    state.remote.lock().unwrap().start(app, ip, port)
}
#[tauri::command]
pub fn remote_stop(state: tauri::State<AppState>) {
    state.remote.lock().unwrap().stop();
}
/// Android transport: Rust performs LAN HTTP, avoiding WebView mixed content/CORS.
#[tauri::command(async)]
pub fn remote_request(
    address: String,
    token: String,
    request: protocol::Request,
) -> Result<serde_json::Value, String> {
    let addr = protocol::client_address(&address)?;
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("remote_auth".into());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(3))
        .redirects(0)
        .try_proxy_from_env(false)
        .build();
    let response = agent
        .post(&format!("http://{addr}/api"))
        .set("Authorization", &format!("Bearer {token}"))
        .send_json(serde_json::to_value(request).map_err(|_| "remote_command")?)
        .map_err(|e| match e {
            ureq::Error::Status(403, _) => "remote_auth",
            _ => "remote_connection",
        })?;
    use std::io::Read;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|_| "remote_connection")?;
    if bytes.len() > 1_048_576 {
        return Err("remote_connection".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "remote_connection".into())
}
