//! Filtros y operaciones de grabación: validación y estado confirmado por OBS.
use crate::model::obs::{ObsAction, ObsSnapshot};
use serde_json::{json, Value};

pub fn handles(kind: &str) -> bool {
    matches!(
        kind,
        "filter_toggle"
            | "filter_on"
            | "filter_off"
            | "record_pause_toggle"
            | "record_pause"
            | "record_resume"
            | "record_split"
    )
}
pub fn validate(a: &ObsAction) -> Result<(), String> {
    let filter = matches!(
        a.kind.as_str(),
        "filter_toggle" | "filter_on" | "filter_off"
    );
    if (filter && (a.target.is_empty() || a.filter.is_empty()))
        || a.filter.len() > 1024
        || (!filter && !a.filter.is_empty())
        || (handles(&a.kind)
            && (a.duration_ms.is_some() || !a.scene.is_empty() || a.scene_item_id.is_some()))
        || (handles(&a.kind) && !filter && !a.target.is_empty())
    {
        return Err("obs_invalid_action".into());
    }
    Ok(())
}
pub fn request(a: &ObsAction, s: &ObsSnapshot) -> Result<(&'static str, Value), String> {
    if a.kind.starts_with("filter_") {
        let filter = s
            .filters
            .iter()
            .find(|f| f.source == a.target && f.name == a.filter)
            .ok_or("obs_target_unavailable")?;
        let enabled = match a.kind.as_str() {
            "filter_on" => true,
            "filter_off" => false,
            _ => !filter.enabled,
        };
        return Ok((
            "SetSourceFilterEnabled",
            json!({"sourceName":a.target,"filterName":a.filter,"filterEnabled":enabled}),
        ));
    }
    if !s.recording {
        return Err("obs_record_inactive".into());
    }
    let method = match a.kind.as_str() {
        "record_pause" => "PauseRecord",
        "record_resume" => "ResumeRecord",
        "record_pause_toggle" => {
            if s.recording_paused {
                "ResumeRecord"
            } else {
                "PauseRecord"
            }
        }
        "record_split" if s.recording_paused => return Err("obs_record_split_paused".into()),
        "record_split" => "SplitRecordFile",
        _ => return Err("obs_invalid_action".into()),
    };
    Ok((method, json!({})))
}
pub fn state(a: &ObsAction, s: &ObsSnapshot) -> &'static str {
    if a.kind.starts_with("filter_") {
        return match s
            .filters
            .iter()
            .find(|f| f.source == a.target && f.name == a.filter)
        {
            Some(f) if f.enabled => "filter_on",
            Some(_) => "filter_off",
            None => "unavailable",
        };
    }
    let method = if a.kind == "record_split" {
        "SplitRecordFile"
    } else if a.kind == "record_resume" || (a.kind == "record_pause_toggle" && s.recording_paused) {
        "ResumeRecord"
    } else {
        "PauseRecord"
    };
    if !s.available.iter().any(|v| v == method) {
        return "unavailable";
    }
    if !s.recording {
        "recording_stopped"
    } else if s.recording_paused {
        "recording_paused"
    } else if a.kind == "record_split" {
        "split_ready"
    } else {
        "recording"
    }
}
