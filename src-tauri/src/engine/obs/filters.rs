//! Consulta compartida de filtros de entradas, escenas y grupos, sin duplicados.
use super::{sync::bool_field, transport::Session};
use crate::model::obs::{ObsFilter, ObsSnapshot};
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn refresh(
    session: &mut Session,
    state: &mut ObsSnapshot,
    mut sources: Vec<String>,
) -> Result<(), String> {
    let mut filters = Vec::new();
    if ["GetSourceFilterList", "SetSourceFilterEnabled"]
        .iter()
        .all(|name| state.available.iter().any(|v| v == name))
    {
        sources.extend(state.scenes.iter().cloned());
        sources.extend(
            state
                .scene_items
                .iter()
                .flat_map(|i| [i.scene.clone(), i.name.clone()]),
        );
        let mut visited = HashSet::new();
        for source in sources {
            if !visited.insert(source.clone()) {
                continue;
            }
            let Some(data) = session.request_optional(
                "GetSourceFilterList",
                json!({"sourceName":source}),
                &[600],
            )?
            else {
                continue;
            };
            filters.extend(parse(&source, &data)?);
        }
    }
    state.filters = filters;
    Ok(())
}
fn parse(source: &str, data: &Value) -> Result<Vec<ObsFilter>, String> {
    data["filters"]
        .as_array()
        .ok_or("obs_protocol_error")?
        .iter()
        .map(|v| {
            Ok(ObsFilter {
                source: source.into(),
                name: v["filterName"].as_str().ok_or("obs_protocol_error")?.into(),
                enabled: bool_field(v, "filterEnabled")?,
            })
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::super::test_support::mock;
    use super::*;
    #[test]
    fn catalogs_share_input_list_and_query_each_source_only_once() {
        let (mut session, server) = mock(vec![
            (
                "GetInputList",
                json!({}),
                Ok(json!({"inputs":[{"inputName":"Camera"},{"inputName":"Camera"}]})),
            ),
            (
                "GetSourceFilterList",
                json!({"sourceName":"Camera"}),
                Ok(json!({"filters":[{"filterName":"Color","filterEnabled":true}]})),
            ),
            (
                "GetSourceFilterList",
                json!({"sourceName":"Main"}),
                Ok(json!({"filters":[]})),
            ),
        ]);
        let mut state = ObsSnapshot {
            scenes: vec!["Main".into()],
            available: [
                "GetInputList",
                "GetSourceFilterList",
                "SetSourceFilterEnabled",
            ]
            .map(String::from)
            .to_vec(),
            ..Default::default()
        };
        super::super::resources::refresh(&mut session, &mut state).unwrap();
        assert_eq!(state.filters.len(), 1);
        assert_eq!(state.filters[0].source, "Camera");
        server.join().unwrap();
    }
    #[test]
    fn same_filter_name_keeps_source_identity_and_unknown_is_not_disabled() {
        let value = json!({"filters":[{"filterName":"Color","filterEnabled":false}]});
        assert_eq!(parse("Camera", &value).unwrap()[0].source, "Camera");
        assert_eq!(parse("Scene", &value).unwrap()[0].source, "Scene");
        assert!(!parse("Camera", &value).unwrap()[0].enabled);
        assert!(parse("Camera", &json!({"filters":[{"filterName":"Color"}]})).is_err());
        assert!(parse("Camera", &Value::Null).is_err());
    }
}
