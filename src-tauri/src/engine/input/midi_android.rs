use super::midi::MidiRuntimeMsg;
use super::midi_message;
use super::midi_ports::MidiPortInfo;
use super::midi_stream::MidiStream;
use crate::core::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{mpsc::Sender, Mutex};
use tauri::ipc::Channel;
use tauri::plugin::mobile::PluginInvokeError;
use tauri::plugin::{Builder, PluginHandle, TauriPlugin};
use tauri::{AppHandle, Emitter, Manager, Wry};

type NativeMidi = PluginHandle<Wry>;

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new("android-midi")
        .setup(|app, api| {
            let native = api.register_android_plugin("lf.botonera.midi", "MidiPlugin")?;
            app.manage(native);
            Ok(())
        })
        .build()
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NativeEvent {
    Devices {
        ports: Vec<MidiPortInfo>,
        connected: Vec<String>,
    },
    Bytes {
        id: String,
        name: String,
        stream: u32,
        data: Vec<u8>,
    },
    Closed {
        id: String,
    },
}

pub(super) fn attach(app: &AppHandle, tx: Sender<MidiRuntimeMsg>) -> impl Fn(&AppHandle) {
    let receiver_app = app.clone();
    let streams = Mutex::new(HashMap::<(String, u32), MidiStream>::new());
    let channel = Channel::<Value>::new(move |body| {
        match body.deserialize::<NativeEvent>()? {
            NativeEvent::Devices { ports, connected } => {
                receiver_app
                    .state::<AppState>()
                    .midi
                    .replace_android_devices(ports, connected);
                let _ = receiver_app.emit("midi-devices-changed", ());
            }
            NativeEvent::Bytes {
                id,
                name,
                stream,
                data,
            } => {
                let mut streams = streams.lock().unwrap();
                streams
                    .entry((id.clone(), stream))
                    .or_default()
                    .feed(&data, |message| {
                        if let Some(event) = midi_message::parse(&id, &name, message) {
                            let _ = tx.send(MidiRuntimeMsg::Incoming(event));
                        }
                    });
            }
            NativeEvent::Closed { id } => streams
                .lock()
                .unwrap()
                .retain(|(source, _), _| source != &id),
        }
        Ok(())
    });
    if let Err(error) = app
        .state::<NativeMidi>()
        .run_mobile_plugin::<()>("attach", json!({ "channel": channel }))
    {
        eprintln!("Android MIDI attach: {error}");
    }
    sync
}

fn sync(app: &AppHandle) {
    let config = app.state::<AppState>().config.lock().unwrap().midi.clone();
    let ids = if config.enabled {
        config
            .inputs
            .into_iter()
            .map(|device| device.id)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if let Err(error) = app
        .state::<NativeMidi>()
        .run_mobile_plugin::<()>("select", json!({ "ids": ids }))
    {
        eprintln!("Android MIDI selection: {error}");
    }
}

pub async fn scan(app: &AppHandle) -> Result<usize, String> {
    app.state::<NativeMidi>()
        .run_mobile_plugin_async::<usize>("scanBluetooth", ())
        .await
        .map_err(|error| {
            if let PluginInvokeError::InvokeRejected(response) = error {
                if let Some(code) = response.code {
                    return code;
                }
            }
            "bluetooth_unavailable".into()
        })
}
