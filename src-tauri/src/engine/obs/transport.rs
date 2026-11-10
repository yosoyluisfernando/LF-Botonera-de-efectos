//! WebSocket síncrono, limitado por tiempos y tamaño; solo lo usa el worker OBS.
use crate::model::obs::ObsConfig;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket};

pub struct Session {
    socket: WebSocket<TcpStream>,
    next_id: u64,
}
impl Session {
    pub fn connect(cfg: &ObsConfig, password: &str) -> Result<Self, String> {
        let addresses = (cfg.host.as_str(), cfg.port)
            .to_socket_addrs()
            .map_err(|_| "obs_connection_failed")?;
        let stream = addresses
            .take(4)
            .find_map(|a| TcpStream::connect_timeout(&a, Duration::from_millis(750)).ok())
            .ok_or("obs_connection_failed")?;
        stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|_| "obs_connection_failed")?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| "obs_connection_failed")?;
        let host = if cfg.host.contains(':') {
            format!("[{}]", cfg.host)
        } else {
            cfg.host.clone()
        };
        let settings = tungstenite::protocol::WebSocketConfig::default()
            .max_message_size(Some(2 * 1024 * 1024))
            .max_frame_size(Some(2 * 1024 * 1024));
        let (socket, _) = tungstenite::client::client_with_config(
            format!("ws://{host}:{}/", cfg.port).as_str(),
            stream,
            Some(settings),
        )
        .map_err(|_| "obs_connection_failed")?;
        let mut s = Self { socket, next_id: 0 };
        let hello = s.wait_op(0)?;
        if hello["d"]["rpcVersion"].as_u64().unwrap_or(0) < 1 {
            return Err("obs_unsupported".into());
        }
        let mut identify = json!({"rpcVersion":1,"eventSubscriptions":2047});
        if let Some(auth) = hello["d"].get("authentication") {
            let salt = auth["salt"].as_str().ok_or("obs_protocol_error")?;
            let challenge = auth["challenge"].as_str().ok_or("obs_protocol_error")?;
            identify["authentication"] = json!(authentication(password, salt, challenge));
        }
        s.send(json!({"op":1,"d":identify}))?;
        s.wait_op(2).map_err(|_| "obs_auth_failed".to_string())?;
        Ok(s)
    }
    fn send(&mut self, value: Value) -> Result<(), String> {
        self.socket
            .send(Message::Text(value.to_string().into()))
            .map_err(|_| "obs_disconnected".into())
    }
    pub fn read(&mut self) -> Result<Option<Value>, String> {
        match self.socket.read() {
            Ok(Message::Text(v)) => serde_json::from_str(&v)
                .map(Some)
                .map_err(|_| "obs_protocol_error".into()),
            Ok(Message::Close(_)) => Err("obs_disconnected".into()),
            Ok(_) => Ok(None),
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                Ok(None)
            }
            Err(_) => Err("obs_disconnected".into()),
        }
    }
    fn wait_op(&mut self, op: u64) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Some(v) = self.read()? {
                if v["op"] == op {
                    return Ok(v);
                }
            }
        }
        Err("obs_timeout".into())
    }
    pub fn request(&mut self, kind: &str, data: Value) -> Result<Value, String> {
        self.request_optional(kind, data, &[])?
            .ok_or("obs_protocol_error".into())
    }
    /// Solo se omiten rechazos de catálogo conocidos; nunca fallos de transporte.
    pub fn request_optional(
        &mut self,
        kind: &str,
        data: Value,
        absent: &[u64],
    ) -> Result<Option<Value>, String> {
        self.next_id += 1;
        let id = self.next_id.to_string();
        self.send(json!({"op":6,"d":{"requestType":kind,"requestId":id,"requestData":data}}))?;
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Some(v) = self.read()? {
                if v["op"] == 7 && v["d"]["requestId"] == id {
                    if v["d"]["requestStatus"]["result"] != true {
                        if v["d"]["requestStatus"]["code"]
                            .as_u64()
                            .is_some_and(|code| absent.contains(&code))
                        {
                            return Ok(None);
                        }
                        return Err("obs_request_failed".into());
                    }
                    return Ok(Some(v["d"]["responseData"].clone()));
                }
            }
        }
        Err("obs_timeout".into())
    }
}
pub fn authentication(password: &str, salt: &str, challenge: &str) -> String {
    let secret = STANDARD.encode(Sha256::digest(format!("{password}{salt}")));
    STANDARD.encode(Sha256::digest(format!("{secret}{challenge}")))
}
#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
