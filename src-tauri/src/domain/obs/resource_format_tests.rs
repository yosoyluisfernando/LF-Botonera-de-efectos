use super::actions;
use crate::model::obs::ObsAction;
fn action(kind: &str) -> ObsAction {
    ObsAction {
        kind: kind.into(),
        ..Default::default()
    }
}
#[test]
fn old_actions_load_without_new_fields_and_new_identity_survives_shared_format() {
    let old: ObsAction =
        serde_json::from_str(r#"{"kind":"scene_program","target":"Main"}"#).unwrap();
    assert_eq!(old.scene, "");
    assert_eq!(old.scene_item_id, None);
    assert!(actions::validate(&old).is_ok());
    use crate::domain::{
        button::defaults::new_button,
        export::lfa_format::{from_lfa_paleta, to_lfa_paleta},
    };
    let mut p = crate::model::AppConfig::default()
        .profiles
        .remove(0)
        .paletas
        .remove(0);
    p.obs = true;
    let mut b = new_button(&p.id, 1, "Logo", "#000", "#fff");
    b.type_field = "obs".into();
    b.obs = Some(ObsAction {
        target: "Logo".into(),
        scene: "Main".into(),
        scene_item_id: Some(4),
        ..action("item_toggle")
    });
    p.botones.push(b);
    let json = serde_json::to_string(&to_lfa_paleta(&p)).unwrap();
    let restored = from_lfa_paleta(serde_json::from_str(&json).unwrap(), "restored".into());
    assert_eq!(restored.botones[0].obs, p.botones[0].obs);
}
