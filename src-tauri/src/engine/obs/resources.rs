//! Catálogos y estados OBS opcionales. Cada catálogo se reemplaza completo.
use super::{sync::bool_field, transport::Session};
use crate::model::obs::{ObsInput, ObsSceneItem, ObsSnapshot};
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn refresh(session: &mut Session, state: &mut ObsSnapshot) -> Result<(), String> {
    let has = |name: &str| state.available.iter().any(|v| v == name);
    let audio = has("GetInputList") && has("GetInputMute") && has("SetInputMute");
    let filters = has("GetSourceFilterList") && has("SetSourceFilterEnabled");
    // Una sola enumeración, compartida por audio y filtros; incluye fuentes sin audio.
    let sources = if has("GetInputList") && (audio || filters) {
        let list = session.request("GetInputList", json!({}))?;
        array(&list["inputs"])?
            .iter()
            .map(|v| text(v, "inputName").map(String::from))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let mut inputs = Vec::new();
    if audio {
        for name in &sources {
            // 604: entrada sin audio; 600: eliminada entre listado y consulta.
            if let Some(status) =
                session.request_optional("GetInputMute", json!({"inputName":name}), &[600, 604])?
            {
                inputs.push(ObsInput {
                    name: name.into(),
                    muted: bool_field(&status, "inputMuted")?,
                });
            }
        }
    }
    let mut items = Vec::new();
    if has("GetSceneItemList") && has("SetSceneItemEnabled") {
        let mut pending = state
            .scenes
            .iter()
            .rev()
            .map(|name| (name.clone(), false))
            .collect::<Vec<_>>();
        let mut visited = HashSet::new();
        while let Some((scene, group)) = pending.pop() {
            if !visited.insert(scene.clone()) {
                continue;
            }
            if group && !has("GetGroupSceneItemList") {
                continue;
            }
            let method = if group {
                "GetGroupSceneItemList"
            } else {
                "GetSceneItemList"
            };
            let Some(data) =
                session.request_optional(method, json!({"sceneName":scene}), &[600])?
            else {
                continue;
            };
            let mut values = array(&data["sceneItems"])?.to_vec();
            values.sort_by_key(|v| std::cmp::Reverse(v["sceneItemIndex"].as_i64().unwrap_or(0)));
            for value in values {
                let item = parse_item(&scene, &value)?;
                if value["isGroup"] == true {
                    pending.push((item.name.clone(), true));
                }
                items.push(item);
            }
        }
    }
    let replay_active = if has("GetReplayBufferStatus") {
        // OBS puede ofrecer el método aunque el usuario no haya habilitado el búfer.
        session
            .request_optional("GetReplayBufferStatus", json!({}), &[604])?
            .map(|data| bool_field(&data, "outputActive"))
            .transpose()?
    } else {
        None
    };
    state.inputs = inputs;
    state.scene_items = items;
    state.replay_active = replay_active;
    super::filters::refresh(session, state, sources)?;
    Ok(())
}
fn array(v: &Value) -> Result<&Vec<Value>, String> {
    v.as_array().ok_or("obs_protocol_error".into())
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key].as_str().ok_or("obs_protocol_error".into())
}
fn parse_item(scene: &str, v: &Value) -> Result<ObsSceneItem, String> {
    Ok(ObsSceneItem {
        scene: scene.into(),
        id: v["sceneItemId"]
            .as_i64()
            .filter(|id| *id >= 0)
            .ok_or("obs_protocol_error")?,
        name: text(v, "sourceName")?.into(),
        enabled: bool_field(v, "sceneItemEnabled")?,
    })
}

#[cfg(test)]
#[path = "resources_tests.rs"]
mod tests;
