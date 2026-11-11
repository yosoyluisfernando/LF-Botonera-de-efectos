use super::*;

fn port(id: &str, name: &str) -> MidiPortInfo {
    MidiPortInfo {
        id: id.into(),
        name: name.into(),
        #[cfg(not(target_os = "android"))]
        index: 0,
        #[cfg(target_os = "linux")]
        native_id: String::new(),
        #[cfg(target_os = "android")]
        connection_state: "failed".into(),
    }
}

#[test]
fn bluetooth_address_does_not_replace_a_known_name() {
    for id in ["ble:AA:BB:CC:DD:EE:FF", "ble-desktop:AA:BB:CC:DD:EE:FF"] {
        for candidate in ["AA:BB:CC:DD:EE:FF", "", "   ", id] {
            assert_eq!(
                preferred_name(id, candidate, Some("My controller")),
                "My controller"
            );
        }
        assert_eq!(preferred_name(id, "New name", Some("Old name")), "New name");
    }
}

#[test]
fn missing_name_keeps_an_identifier_without_inventing_a_name() {
    assert_eq!(preferred_name("ble:AA:BB", "", None), "ble:AA:BB");
    assert_eq!(preferred_name("ble:AA:BB", "AA:BB", None), "AA:BB");
}

#[test]
fn device_list_uses_saved_bluetooth_name_and_live_connection() {
    let id = "ble:AA:BB:CC:DD:EE:FF";
    let config = MidiConfig {
        enabled: true,
        inputs: vec![MidiDeviceRef {
            id: id.into(),
            name: "Controller".into(),
        }],
    };
    let ports = vec![port(id, "AA:BB:CC:DD:EE:FF")];
    let devices = device_view(&config, &[id.into()], &ports);
    assert_eq!(devices[0].name, "Controller");
    assert_eq!(devices[0].connection_state, "connected");
    assert!(devices[0].selected && devices[0].connected);
    let missing = device_view(&config, &[], &[]);
    assert_eq!(missing[0].name, "Controller");
    assert_eq!(missing[0].connection_state, "disconnected");
    assert!(!missing[0].available && !missing[0].connected);
}

#[test]
fn a_discovered_new_name_takes_precedence_over_saved_name() {
    let config = MidiConfig {
        enabled: true,
        inputs: vec![MidiDeviceRef {
            id: "ble:AA:BB".into(),
            name: "Old".into(),
        }],
    };
    let devices = device_view(&config, &[], &[port("ble:AA:BB", "New")]);
    assert_eq!(devices[0].name, "New");
}
