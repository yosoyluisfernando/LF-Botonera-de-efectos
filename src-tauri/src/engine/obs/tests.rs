use super::*;
use crate::model::obs::ObsAction;
#[test]
fn only_one_action_can_be_pending_and_disconnect_never_queues() {
    let engine = ObsEngine::new();
    let (tx, rx) = sync_channel(8);
    *engine.tx.lock().unwrap() = Some(tx);
    let a = ObsAction {
        kind: "record_start".into(),
        ..Default::default()
    };
    assert_eq!(
        engine.execute("one".into(), a.clone()).unwrap_err(),
        "obs_disconnected"
    );
    assert!(rx.try_recv().is_err());
    *engine.snapshot.lock().unwrap() = ObsSnapshot {
        connected: true,
        available: vec!["StartRecord".into()],
        ..Default::default()
    };
    engine.execute("one".into(), a.clone()).unwrap();
    assert_eq!(engine.execute("two".into(), a).unwrap_err(), "obs_busy");
    assert!(matches!(rx.try_recv().unwrap(),Command::Execute(_,id,_) if id=="one"));
    assert!(rx.try_recv().is_err());
}
#[test]
fn reconfiguration_invalidates_old_actions_and_retains_only_session_secret() {
    let engine = ObsEngine::new();
    let (tx, rx) = sync_channel(8);
    *engine.tx.lock().unwrap() = Some(tx);
    *engine.snapshot.lock().unwrap() = ObsSnapshot {
        connected: true,
        available: vec!["StartRecord".into()],
        ..Default::default()
    };
    engine
        .execute(
            "one".into(),
            ObsAction {
                kind: "record_start".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let Command::Execute(old_ticket, _, _) = rx.try_recv().unwrap() else {
        panic!()
    };
    engine
        .configure(ObsConfig::default(), Some("session-only".into()))
        .unwrap();
    assert_ne!(old_ticket, engine.generation.load(Ordering::SeqCst));
    assert!(!engine.snapshot().connected);
    assert!(!engine.snapshot().busy);
    assert_eq!(engine.session_password(), "session-only");
    assert!(!serde_json::to_string(&ObsConfig::default())
        .unwrap()
        .contains("session-only"));
}
