use super::{assets, http, protocol};
use serde::Serialize;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tauri::Manager;

pub struct Session {
    pub alive: AtomicBool,
    pub gate: Mutex<()>,
    pub token: String,
    workers: AtomicUsize,
}
#[derive(Default)]
pub struct RemoteServer {
    session: Option<Arc<Session>>,
    address: Option<SocketAddr>,
    thread: Option<std::thread::JoinHandle<()>>,
}
#[derive(Serialize)]
pub struct RemoteStatus {
    pub enabled: bool,
    pub url: String,
    pub token: String,
    pub suggested_ip: String,
}
impl RemoteServer {
    pub fn status(&self) -> RemoteStatus {
        RemoteStatus {
            enabled: self.session.is_some(),
            url: self
                .address
                .map(|addr| format!("http://{addr}"))
                .unwrap_or_default(),
            suggested_ip: suggested_ip(),
            token: self
                .session
                .as_ref()
                .map(|s| s.token.clone())
                .unwrap_or_default(),
        }
    }
    pub fn start(
        &mut self,
        app: tauri::AppHandle,
        ip: String,
        port: u16,
    ) -> Result<RemoteStatus, String> {
        if cfg!(target_os = "android") || port < 1024 {
            return Err("remote_address".into());
        }
        if self.session.is_some() {
            return Ok(self.status());
        }
        let address = protocol::address(&format!("{}:{port}", ip.trim()))?;
        let listener = TcpListener::bind(address).map_err(|_| "remote_bind".to_string())?;
        listener.set_nonblocking(true).map_err(|_| "remote_bind")?;
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| "remote_start")?;
        let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        let session = Arc::new(Session {
            alive: AtomicBool::new(true),
            gate: Mutex::new(()),
            token,
            workers: AtomicUsize::new(0),
        });
        let run = session.clone();
        let thread = std::thread::Builder::new()
            .name("remote-lan".into())
            .spawn(move || {
                while run.alive.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, peer)) => {
                            if !protocol::lan_ip(peer.ip())
                                || run.workers.load(Ordering::Relaxed) >= 8
                            {
                                continue;
                            }
                            run.workers.fetch_add(1, Ordering::Relaxed);
                            let (session, app) = (run.clone(), app.clone());
                            let worker_session = session.clone();
                            if std::thread::Builder::new()
                                .name("remote-request".into())
                                .spawn(move || {
                                    serve(stream, address, &app, &worker_session);
                                    worker_session.workers.fetch_sub(1, Ordering::Relaxed);
                                })
                                .is_err()
                            {
                                session.workers.fetch_sub(1, Ordering::Relaxed);
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(20));
                        }
                        Err(_) => break,
                    }
                }
            })
            .map_err(|_| "remote_start")?;
        self.address = Some(address);
        self.session = Some(session);
        self.thread = Some(thread);
        Ok(self.status())
    }
    pub fn stop(&mut self) {
        self.address = None;
        if let Some(session) = self.session.take() {
            let _gate = session.gate.lock().unwrap();
            session.alive.store(false, Ordering::Release);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for RemoteServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn suggested_ip() -> String {
    UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .and_then(|s| {
            s.connect("192.168.0.1:9")?;
            s.local_addr()
        })
        .ok()
        .map(|a| a.ip())
        .filter(|ip| protocol::lan_ip(*ip))
        .map(|ip| ip.to_string())
        .unwrap_or_default()
}

fn serve(mut stream: TcpStream, address: SocketAddr, app: &tauri::AppHandle, session: &Session) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let Ok(request) = http::read(&mut stream, address) else {
        http::reply(&mut stream, 400, "text/plain", b"bad_request");
        return;
    };
    if !session.alive.load(Ordering::Acquire) {
        http::reply(&mut stream, 403, "text/plain", b"remote_disabled");
        return;
    }
    if request.method == "GET" {
        if let Some((mime, data)) = assets::get(&request.path) {
            http::reply(&mut stream, 200, mime, data);
        } else {
            http::reply(&mut stream, 404, "text/plain", b"not_found");
        }
        return;
    }
    if request.method != "POST" || request.path != "/api" {
        http::reply(&mut stream, 404, "text/plain", b"not_found");
        return;
    }
    if request.authorization != format!("Bearer {}", session.token) {
        http::reply(
            &mut stream,
            403,
            "application/json",
            br#"{"error":"remote_auth"}"#,
        );
        return;
    }
    let gate = session.gate.lock().unwrap();
    if !session.alive.load(Ordering::Acquire) {
        drop(gate);
        http::reply(&mut stream, 403, "text/plain", b"remote_disabled");
        return;
    }
    let result = serde_json::from_slice::<protocol::Request>(&request.body)
        .map_err(|_| "remote_command".to_string())
        .and_then(|r| protocol::execute(&app.state::<crate::core::AppState>(), r));
    drop(gate);
    let (status, value) = match result {
        Ok(v) => (200, v),
        Err(e) => (400, serde_json::json!({"error": e})),
    };
    let data = serde_json::to_vec(&value).unwrap_or_default();
    http::reply(&mut stream, status, "application/json", &data);
}
