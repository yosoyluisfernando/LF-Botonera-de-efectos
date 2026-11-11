use super::SERVICE;
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager as BluetoothManager};
use std::time::Duration;
use tokio::time::timeout;

pub(super) async fn start() -> Result<Vec<Adapter>, String> {
    let adapters = timeout(Duration::from_secs(5), async {
        BluetoothManager::new().await?.adapters().await
    })
    .await
    .map_err(|_| "bluetooth_scan_failed")?
    .map_err(|_| "bluetooth_unavailable")?;
    if adapters.is_empty() {
        return Err("bluetooth_unavailable".into());
    }
    let mut running = Vec::new();
    for adapter in adapters {
        let filter = ScanFilter {
            services: vec![SERVICE],
        };
        if matches!(
            timeout(Duration::from_secs(4), adapter.start_scan(filter)).await,
            Ok(Ok(()))
        ) {
            running.push(adapter);
        }
    }
    if running.is_empty() {
        return Err("bluetooth_scan_failed".into());
    }
    Ok(running)
}
