//! Las asignaciones conocidas se resuelven por ID; solo las antiguas usan nombre único.
pub fn resolve<'a>(
    name: &str,
    id: &str,
    devices: &'a [(String, String)],
) -> Option<&'a (String, String)> {
    if !id.is_empty() {
        return devices.iter().find(|(key, _)| key == id);
    }
    let mut matches = devices.iter().filter(|(_, label)| label == name);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

pub fn set_selection(tab: &mut crate::model::PaletaData, name: Option<String>, id: Option<String>) {
    if let Some(name) = name {
        if tab.audio_out != name {
            tab.audio_out_id.clear();
        }
        tab.audio_out = name;
    }
    if let Some(id) = id {
        tab.audio_out_id = id;
    }
    if tab.audio_out.is_empty() || tab.audio_out == "default" {
        tab.audio_out_id.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn list() -> Vec<(String, String)> {
        vec![
            ("id-a".into(), "Altavoces".into()),
            ("id-b".into(), "USB".into()),
        ]
    }
    #[test]
    fn android_keeps_reserved_outputs_disabled() {
        for bus in crate::domain::console::BusId::salidas() {
            assert!(!bus.wants_output(&crate::domain::console::routing::single_output(bus)));
        }
    }
    #[test]
    fn legacy_unique_name_is_upgraded() {
        let d = list();
        assert_eq!(resolve("USB", "", &d).unwrap().0, "id-b");
    }
    #[test]
    fn rename_keeps_identity() {
        let d = list();
        assert_eq!(resolve("Nombre anterior", "id-b", &d).unwrap().1, "USB");
    }
    #[test]
    fn missing_identity_never_selects_a_replacement_with_same_name() {
        assert!(resolve("USB", "old-id", &list()).is_none());
    }
    #[test]
    fn duplicate_names_are_not_guessed() {
        let mut d = list();
        d.push(("id-c".into(), "USB".into()));
        assert!(resolve("USB", "", &d).is_none());
        assert_eq!(resolve("USB", "id-c", &d).unwrap().0, "id-c");
    }
    #[test]
    fn stable_id_and_name_survive_shared_format() {
        let cfg = crate::model::AppConfig::default();
        let mut tab = cfg.active_paleta().unwrap().clone();
        tab.audio_out = "USB".into();
        tab.audio_out_id = "wasapi:opaque".into();
        let shared = crate::domain::export::lfa_format::to_lfa_paleta(&tab);
        let value = serde_json::to_value(&shared).unwrap();
        assert_eq!(value["audioOut"], "USB");
        assert_eq!(value["audioOutId"], "wasapi:opaque");
        let restored =
            crate::domain::export::lfa_format::from_lfa_paleta(shared, "imported".into());
        assert_eq!(restored.audio_out_id, tab.audio_out_id);
        tab.audio_out_id.clear();
        assert!(serde_json::to_value(&tab)
            .unwrap()
            .get("audio_out_id")
            .is_none());
    }
}
