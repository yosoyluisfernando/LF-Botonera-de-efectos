use super::transport::Session;
use crate::{
    domain::obs::actions,
    model::obs::{ObsAction, ObsSnapshot},
};
use serde_json::{json, Value};

pub fn refresh(s: &mut Session, state: &mut ObsSnapshot) -> Result<(), String> {
    let version = s.request("GetVersion", json!({}))?;
    state.available = names(&version["availableRequests"], None)?;
    state.quick_transitions.clear();
    if ["GetHotkeyList", "TriggerHotkeyByName"]
        .iter()
        .all(|name| state.available.iter().any(|n| n == name))
    {
        let hotkeys = s.request("GetHotkeyList", json!({}))?;
        state.quick_transitions = actions::quick_transitions(&names(&hotkeys["hotkeys"], None)?);
    }
    let scenes = s.request("GetSceneList", json!({}))?;
    state.scenes = scene_names(&scenes["scenes"])?;
    state.program = scenes["currentProgramSceneName"]
        .as_str()
        .unwrap_or("")
        .into();
    state.preview = scenes["currentPreviewSceneName"]
        .as_str()
        .unwrap_or("")
        .into();
    state.studio = bool_field(
        &s.request("GetStudioModeEnabled", json!({}))?,
        "studioModeEnabled",
    )?;
    super::record::refresh(s, state)?;
    state.streaming = bool_field(&s.request("GetStreamStatus", json!({}))?, "outputActive")?;
    let transitions = s.request("GetSceneTransitionList", json!({}))?;
    state.transitions = names(&transitions["transitions"], Some("transitionName"))?;
    let current = s.request("GetCurrentSceneTransition", json!({}))?;
    state.transition = current["transitionName"].as_str().unwrap_or("").into();
    state.fixed_duration = bool_field(&current, "transitionFixed")?;
    super::resources::refresh(s, state)?;
    Ok(())
}
fn names(v: &Value, key: Option<&str>) -> Result<Vec<String>, String> {
    v.as_array()
        .ok_or("obs_protocol_error")?
        .iter()
        .map(|v| {
            key.map_or(v, |k| &v[k])
                .as_str()
                .map(String::from)
                .ok_or("obs_protocol_error".into())
        })
        .collect()
}
// obs-websocket invierte la enumeración de OBS: sceneIndex mayor es la primera
// escena visible. Reconstruir el orden de la interfaz, no el alfabético.
fn scene_names(v: &Value) -> Result<Vec<String>, String> {
    let mut scenes = v
        .as_array()
        .ok_or("obs_protocol_error")?
        .iter()
        .map(|scene| {
            let index = scene["sceneIndex"].as_u64().ok_or("obs_protocol_error")?;
            let name = scene["sceneName"].as_str().ok_or("obs_protocol_error")?;
            Ok((index, name.to_string()))
        })
        .collect::<Result<Vec<_>, String>>()?;
    scenes.sort_by_key(|(index, _)| std::cmp::Reverse(*index));
    Ok(scenes.into_iter().map(|(_, name)| name).collect())
}
pub(super) fn bool_field(v: &Value, key: &str) -> Result<bool, String> {
    v[key].as_bool().ok_or("obs_protocol_error".into())
}
pub fn execute(s: &mut Session, a: &ObsAction, state: &ObsSnapshot) -> Result<(), String> {
    let (name, data) = actions::request(a, state)?;
    if a.kind == "transition_select" && a.duration_ms.is_some() {
        // Validar que la versión permite ajustar duración antes de seleccionar.
        if !state
            .available
            .iter()
            .any(|v| v == "SetCurrentSceneTransitionDuration")
        {
            return Err("obs_unsupported".into());
        }
    }
    if super::record::execute(s, a, state, name, data.clone())? {
        return Ok(());
    }
    s.request(name, data)?;
    if a.kind == "transition_select" {
        if let Some(duration) = a.duration_ms {
            let current = s.request("GetCurrentSceneTransition", json!({}))?;
            if current["transitionFixed"] == true {
                return Err("obs_fixed_transition".into());
            }
            s.request(
                "SetCurrentSceneTransitionDuration",
                json!({"transitionDuration":duration}),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_order_matches_obs_dock_instead_of_websocket_enumeration() {
        let scenes = json!([
            {"sceneIndex":0,"sceneName":"PANTALLA NEGRA"},
            {"sceneIndex":2,"sceneName":"PC + Chat"},
            {"sceneIndex":1,"sceneName":"CAM + PC"}
        ]);
        assert_eq!(
            scene_names(&scenes).unwrap(),
            vec!["PC + Chat", "CAM + PC", "PANTALLA NEGRA"]
        );
        assert!(scene_names(&json!([])).unwrap().is_empty());
        assert!(scene_names(&json!([{"sceneName":"Camera"}])).is_err());
    }
    #[test]
    fn unknown_output_state_is_never_interpreted_as_stopped() {
        assert!(bool_field(&json!({}), "outputActive").is_err());
        assert!(bool_field(&json!({"outputActive":"false"}), "outputActive").is_err());
        assert!(!bool_field(&json!({"outputActive":false}), "outputActive").unwrap());
    }
    #[test]
    fn malformed_catalog_is_not_a_valid_empty_catalog() {
        assert!(names(&Value::Null, None).is_err());
        assert!(names(&json!([{}]), Some("sceneName")).is_err());
        assert!(names(&json!([]), None).unwrap().is_empty());
    }
}
