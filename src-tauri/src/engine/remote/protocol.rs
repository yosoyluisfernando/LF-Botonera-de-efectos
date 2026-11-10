use crate::{core::AppState, model::AppConfig};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::{IpAddr, SocketAddr};

#[derive(Deserialize, Serialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Snapshot {},
    Play { palette: String, id: String },
    Stop { id: String },
    StopAll {},
}
pub fn lan_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        IpAddr::V6(_) => false,
    }
}
pub fn address(raw: &str) -> Result<SocketAddr, String> {
    let addr: SocketAddr = raw.parse().map_err(|_| "remote_address")?;
    if !lan_ip(addr.ip()) || addr.port() < 1024 {
        return Err("remote_address".into());
    }
    Ok(addr)
}
pub fn client_address(raw: &str) -> Result<SocketAddr, String> {
    let raw = raw
        .trim()
        .strip_prefix("http://")
        .unwrap_or(raw.trim())
        .trim_end_matches('/');
    address(raw.strip_suffix("/remote.html").unwrap_or(raw))
}
pub fn remote_button<'a>(
    cfg: &'a AppConfig,
    palette: &str,
    id: &str,
) -> Option<&'a crate::model::ButtonData> {
    cfg.active_paleta()
        .filter(|p| p.id == palette && !p.obs)?
        .botones
        .iter()
        .find(|b| b.id == id && is_audio(b))
}
fn is_audio(button: &crate::model::ButtonData) -> bool {
    button.type_field == "audio" && !button.path.is_empty()
}
pub fn execute(state: &AppState, request: Request) -> Result<Value, String> {
    match request {
        Request::Snapshot {} => return Ok(snapshot(state)),
        Request::Play { palette, id } => {
            crate::ipc::cmd_button_playback::play_remote_button(state, &palette, &id).map_err(
                |e| {
                    if e == "remote_stale" {
                        e
                    } else {
                        "remote_play".into()
                    }
                },
            )?;
        }
        Request::Stop { id } => {
            let states = state.audio.lock().unwrap().button_states_handle();
            let states = states.lock().unwrap();
            let effect = states.get(&id).is_some_and(|sources| {
                sources
                    .iter()
                    .any(|s| s.group != crate::engine::audio::button::PlaybackGroup::Cue)
            });
            drop(states);
            if effect {
                crate::ipc::cmd_audio::stop_effect(state, &id);
            }
        }
        Request::StopAll {} => {
            // Mismo transporte que Detener todo local; no toca el reproductor auxiliar.
            crate::ipc::cmd_audio::stop_effects(state);
        }
    }
    Ok(json!({"ok": true}))
}
fn snapshot(state: &AppState) -> Value {
    let cfg = state.config.lock().unwrap();
    let palette = cfg.active_paleta();
    let buttons: Vec<Value> = palette
        .filter(|p| !p.obs)
        .map(|p| {
            p.botones
                .iter()
                .filter(|b| is_audio(b))
                .map(|b| {
                    json!({
                        "id": b.id, "index": b.index, "label": b.label, "name": b.name,
                        "color_bg": b.color_bg, "color_text": b.color_text,
                        "duration_str": b.duration_str,
                        "enabled": true,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let data = json!({
        "palette": palette.map(|p| &p.id), "name": palette.map(|p| &p.nombre),
        "profile": cfg.active_profile().map(|p| &p.name),
        "columns": palette.map(|p| p.cols).unwrap_or(5),
        "rows": palette.map(|p| p.rows).unwrap_or(5), "buttons": buttons,
        "language": cfg.language, "theme": cfg.theme,
        "unavailable": palette.is_some_and(|p| p.obs),
    });
    drop(cfg);
    let handle = state.audio.lock().unwrap().button_states_handle();
    let states = handle.lock().unwrap();
    let playing: Vec<Value> = states
        .iter()
        .filter_map(|(id, sources)| {
            sources
                .iter()
                .rev()
                .find(|s| {
                    !s.is_done() && s.group != crate::engine::audio::button::PlaybackGroup::Cue
                })
                .map(|s| json!({"id": id, "remaining": s.remaining()}))
        })
        .collect();
    let mut data = data;
    data["playing"] = json!(playing);
    data
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restricts_targets_to_local_network_literals() {
        assert_eq!(
            client_address(" http://192.168.1.20:17865/remote.html ").unwrap(),
            address("192.168.1.20:17865").unwrap()
        );
        assert!(client_address("https://192.168.1.20:17865").is_err());
        for s in ["192.168.1.20:17865", "10.0.0.2:17865", "127.0.0.1:17865"] {
            assert!(address(s).is_ok());
        }
        for s in [
            "example.com:17865",
            "8.8.8.8:17865",
            "192.168.1.2:80",
            "[::1]:17865",
            "192.168.1.2:17865/path",
        ] {
            assert!(address(s).is_err());
        }
    }
    #[test]
    fn rejects_arbitrary_ipc_and_paths() {
        for raw in [
            r#"{"command":"play_audio","path":"secret"}"#,
            r#"{"command":"stop_all","path":"secret"}"#,
            r#"{"command":"snapshot","path":"secret"}"#,
        ] {
            assert!(serde_json::from_str::<Request>(raw).is_err());
        }
    }
    #[test]
    fn rejects_stale_or_obs_palettes() {
        let mut cfg = AppConfig::default();
        let button = serde_json::from_value(json!({
            "id": "paleta_1_btn_1", "index": 1, "label": "Test", "path": "local.wav",
            "color_bg": "#123456", "color_text": "#ffffff"
        }))
        .unwrap();
        cfg.profiles[0].paletas[0].botones.push(button);
        assert!(remote_button(&cfg, "paleta_1", "paleta_1_btn_1").is_some());
        assert!(remote_button(&cfg, "old_palette", "paleta_1_btn_1").is_none());
        assert!(remote_button(&cfg, "old_palette", "missing").is_none());
        cfg.profiles[0].paletas[0].obs = true;
        assert!(remote_button(&cfg, "paleta_1", "paleta_1_btn_1").is_none());
    }
}
