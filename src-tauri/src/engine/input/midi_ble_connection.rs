use super::midi::MidiRuntimeMsg;
use super::midi_ble_packet::BleMidiDecoder;
use super::midi_message;
use btleplug::api::{CharPropFlags, Characteristic, Peripheral as _};
use btleplug::platform::Peripheral;
use futures_util::StreamExt;
use std::sync::mpsc::Sender;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

pub(super) const SERVICE: Uuid = Uuid::from_u128(0x03b80e5a_ede8_4b33_a751_6ce34ec4c700);
const DATA: Uuid = Uuid::from_u128(0x7772e5db_3868_4112_a1a9_f2669d106bf3);

pub(super) async fn run(
    device: Peripheral,
    id: String,
    name: String,
    mut enabled: watch::Receiver<bool>,
    tx: Sender<MidiRuntimeMsg>,
    status: impl Fn(bool),
) {
    while enabled.wait_for(|selected| *selected).await.is_ok() {
        let previously_connected = matches!(
            timeout(Duration::from_secs(2), device.is_connected()).await,
            Ok(Ok(true))
        );
        let mut subscription = None;
        tokio::select! {
            _ = enabled.wait_for(|selected| !*selected) => {}
            _ = receive(&device, &id, &name, &tx, &status, &mut subscription) => {}
        }
        status(false);
        if let Some(data) = subscription {
            let _ = timeout(Duration::from_secs(2), device.unsubscribe(&data)).await;
        }
        if !previously_connected {
            let _ = timeout(Duration::from_secs(3), device.disconnect()).await;
        }
        tokio::select! {
            _ = enabled.wait_for(|selected| !*selected) => {}
            _ = sleep(Duration::from_secs(3)) => {}
        }
    }
}

async fn receive(
    device: &Peripheral,
    id: &str,
    name: &str,
    tx: &Sender<MidiRuntimeMsg>,
    status: &impl Fn(bool),
    subscription: &mut Option<Characteristic>,
) -> Option<()> {
    let mut events = timeout(Duration::from_secs(12), async {
        device.connect().await.ok()?;
        device.discover_services().await.ok()?;
        let data = device.characteristics().into_iter().find(|c| {
            c.service_uuid == SERVICE
                && c.uuid == DATA
                && c.properties.contains(CharPropFlags::NOTIFY)
        })?;
        let events = device.notifications().await.ok()?;
        if data.properties.contains(CharPropFlags::READ) {
            let _ = timeout(Duration::from_secs(2), device.read(&data)).await;
        }
        *subscription = Some(data.clone());
        device.subscribe(&data).await.ok()?;
        Some(events)
    })
    .await
    .ok()??;
    status(true);
    let mut decoder = BleMidiDecoder::default();
    let mut poll = tokio::time::interval(Duration::from_secs(2));
    loop {
        tokio::select! {
            packet = events.next() => {
                let packet = packet?;
                if packet.uuid == DATA {
                    decoder.receive(&packet.value, |message| {
                        if let Some(event) = midi_message::parse(id, name, message) {
                            let _ = tx.send(MidiRuntimeMsg::Incoming(event));
                        }
                    });
                }
            }
            _ = poll.tick() => {
                if !timeout(Duration::from_secs(2), device.is_connected()).await.ok()?.ok()? {
                    return None;
                }
            }
        }
    }
}
