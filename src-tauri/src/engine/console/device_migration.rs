//! Migra solo asignaciones inequívocas sin perder nombre ni preferencias anteriores.
use super::device::OutputDevice;
use crate::{domain::console::device_identity, engine::persist::config_io, model::AppConfig};
pub fn apply(config: &std::sync::Mutex<AppConfig>, devices: &[OutputDevice]) -> Result<(), String> {
    let mut cfg = config.lock().unwrap();
    let mut next = cfg.clone();
    if upgrade(&mut next, devices) {
        config_io::save_config(&next)?;
        *cfg = next;
    }
    Ok(())
}
fn upgrade(next: &mut AppConfig, devices: &[OutputDevice]) -> bool {
    let pairs: Vec<_> = devices
        .iter()
        .filter(|d| !d.id.is_empty())
        .map(|d| (d.id.clone(), d.name.clone()))
        .collect();
    let mut changed = false;
    for tab in next
        .profiles
        .iter_mut()
        .flat_map(|p| &mut p.paletas)
        .filter(|p| !p.obs)
    {
        if tab.audio_out.is_empty() || tab.audio_out == "default" {
            continue;
        }
        if let Some((id, name)) =
            device_identity::resolve(&tab.audio_out, &tab.audio_out_id, &pairs)
        {
            if !next.console.output_faders.contains_key(id) {
                if let Some(value) = next.console.output_faders.get(&tab.audio_out).copied() {
                    next.console.output_faders.insert(id.clone(), value);
                    changed = true;
                }
            }
            if tab.audio_out_id != *id || tab.audio_out != *name {
                tab.audio_out_id = id.clone();
                tab.audio_out = name.clone();
                changed = true;
            }
        }
    }
    changed
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_preserves_volume_then_rename_and_missing_device_keep_id() {
        let mut cfg = AppConfig::default();
        cfg.active_paleta_mut().unwrap().audio_out = "USB".into();
        cfg.console.output_faders.insert("USB".into(), 0.35);
        let mut list = vec![OutputDevice {
            id: "wasapi:id".into(),
            name: "USB".into(),
            usable: true,
        }];
        assert!(upgrade(&mut cfg, &list));
        assert_eq!(cfg.active_paleta().unwrap().audio_out_id, "wasapi:id");
        assert_eq!(cfg.console.output_faders["wasapi:id"], 0.35);
        assert!(!upgrade(&mut cfg, &list));
        list[0].name = "Nuevo nombre".into();
        assert!(upgrade(&mut cfg, &list));
        assert_eq!(cfg.active_paleta().unwrap().audio_out, "Nuevo nombre");
        list[0].id = "wasapi:replacement".into();
        assert!(!upgrade(&mut cfg, &list));
        assert_eq!(cfg.active_paleta().unwrap().audio_out_id, "wasapi:id");
    }
    #[test]
    fn duplicate_legacy_name_is_not_pinned() {
        let mut cfg = AppConfig::default();
        cfg.active_paleta_mut().unwrap().audio_out = "USB".into();
        let list = vec![
            OutputDevice {
                id: "a".into(),
                name: "USB".into(),
                usable: false,
            },
            OutputDevice {
                id: "b".into(),
                name: "USB".into(),
                usable: false,
            },
        ];
        assert!(!upgrade(&mut cfg, &list));
        assert!(cfg.active_paleta().unwrap().audio_out_id.is_empty());
    }
}
