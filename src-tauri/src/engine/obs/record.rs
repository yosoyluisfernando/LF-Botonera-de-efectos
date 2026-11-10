//! Grabación real: pausa verificada y división sin detener/iniciar la salida.
use super::{sync::bool_field, transport::Session};
use crate::model::obs::{ObsAction, ObsSnapshot};
use serde_json::{json, Value};
use std::time::Duration;

pub fn refresh(session: &mut Session, state: &mut ObsSnapshot) -> Result<(), String> {
    let status = session.request("GetRecordStatus", json!({}))?;
    apply_status(&status, state)
}
fn apply_status(status: &Value, state: &mut ObsSnapshot) -> Result<(), String> {
    let active = bool_field(status, "outputActive")?;
    let paused = bool_field(status, "outputPaused")?;
    state.recording = active;
    state.recording_paused = active && paused;
    Ok(())
}
pub fn execute(
    session: &mut Session,
    a: &ObsAction,
    state: &ObsSnapshot,
    method: &str,
    data: Value,
) -> Result<bool, String> {
    if !matches!(
        a.kind.as_str(),
        "record_pause" | "record_resume" | "record_pause_toggle" | "record_split"
    ) {
        return Ok(false);
    }
    session.request(method, data).map_err(|e| {
        if a.kind == "record_split" && e == "obs_request_failed" {
            "obs_record_split_unavailable".into()
        } else {
            e
        }
    })?;
    if a.kind != "record_split" {
        let expected = method == "PauseRecord";
        let mut actual = state.clone();
        // La API de OBS puede aceptar una pausa sin que la salida la soporte.
        // Esperar su estado real; el clic y la respuesta de aceptación no bastan.
        for attempt in 0..10 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(50));
            }
            refresh(session, &mut actual)?;
            if !actual.recording {
                return Err("obs_record_inactive".into());
            }
            if actual.recording_paused == expected {
                return Ok(true);
            }
        }
        return Err("obs_record_pause_failed".into());
    }
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::super::test_support::mock;
    use super::*;
    #[test]
    fn accepted_pause_is_not_success_until_output_paused_is_confirmed() {
        let mut steps = vec![("PauseRecord", json!({}), Ok(Value::Null))];
        for _ in 0..10 {
            steps.push((
                "GetRecordStatus",
                json!({}),
                Ok(json!({"outputActive":true,"outputPaused":false})),
            ));
        }
        let (mut session, server) = mock(steps);
        let a = ObsAction {
            kind: "record_pause".into(),
            ..Default::default()
        };
        assert_eq!(
            execute(
                &mut session,
                &a,
                &ObsSnapshot::default(),
                "PauseRecord",
                json!({})
            )
            .unwrap_err(),
            "obs_record_pause_failed"
        );
        server.join().unwrap();
    }
    #[test]
    fn confirmed_resume_and_split_use_direct_requests_without_restart() {
        let (mut session, server) = mock(vec![
            ("ResumeRecord", json!({}), Ok(Value::Null)),
            (
                "GetRecordStatus",
                json!({}),
                Ok(json!({"outputActive":true,"outputPaused":false})),
            ),
            ("SplitRecordFile", json!({}), Ok(Value::Null)),
        ]);
        for (kind, method) in [
            ("record_resume", "ResumeRecord"),
            ("record_split", "SplitRecordFile"),
        ] {
            let a = ObsAction {
                kind: kind.into(),
                ..Default::default()
            };
            assert!(execute(&mut session, &a, &ObsSnapshot::default(), method, json!({})).unwrap());
        }
        server.join().unwrap();
    }
    #[test]
    fn disabled_splitting_has_specific_error_without_retry() {
        let (mut session, server) = mock(vec![("SplitRecordFile", json!({}), Err(702))]);
        let a = ObsAction {
            kind: "record_split".into(),
            ..Default::default()
        };
        assert_eq!(
            execute(
                &mut session,
                &a,
                &ObsSnapshot::default(),
                "SplitRecordFile",
                json!({})
            )
            .unwrap_err(),
            "obs_record_split_unavailable"
        );
        server.join().unwrap();
    }
    #[test]
    fn pause_is_explicit_and_a_stopped_recording_never_looks_paused() {
        let mut state = ObsSnapshot::default();
        assert!(apply_status(&json!({"outputActive":true}), &mut state).is_err());
        apply_status(
            &json!({"outputActive":true,"outputPaused":true}),
            &mut state,
        )
        .unwrap();
        assert!(state.recording_paused);
        apply_status(
            &json!({"outputActive":false,"outputPaused":true}),
            &mut state,
        )
        .unwrap();
        assert!(!state.recording_paused);
    }
}
