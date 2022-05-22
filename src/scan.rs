use std::time::Duration;
use btleplug::api::ScanFilter;
use tokio::time;
use itertools::Itertools;
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::{Manager, Peripheral};
use uuid::Uuid;

lazy_static! {
    static ref TREADMILL_SERVICE_UUID: Uuid = Uuid::parse_str("0000fff0-0000-1000-8000-00805f9b34fb").unwrap();
}

async fn find_peripherals(scan_duration: Duration) -> Result<Vec<Peripheral>, btleplug::Error> {
    let adapters = Manager::new().await?.adapters().await?;
    futures::future::join_all(adapters.into_iter().map(|adapter| async move {
        tracing::debug!("Starting scan on {}", adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices");
        time::sleep(scan_duration).await;
        adapter.peripherals().await
    })).await.into_iter().flatten_ok().collect()
}


pub async fn find_treadmills(scan_duration: Duration) -> Result<Vec<Peripheral>, btleplug::Error> {
    let mut treadmills = Vec::new();
    for peripheral in find_peripherals(scan_duration).await? {
        let properties = peripheral.properties().await?;
        let is_connected = peripheral.is_connected().await?;
        let local_name = properties
            .unwrap()
            .local_name
            .unwrap_or(String::from("(peripheral name unknown)"));
        tracing::debug!(
                "Peripheral {:?} is connected: {:?}",
                local_name, is_connected
            );
        if !is_connected {
            tracing::debug!("Connecting to peripheral {:?}", &local_name);
            if let Err(err) = peripheral.connect().await {
                tracing::warn!("Error connecting to peripheral, skipping: {}", err);
                continue;
            }
        }
        let is_connected = peripheral.is_connected().await?;
        tracing::debug!(
                "Now connected ({:?}) to peripheral {:?}",
                is_connected, &local_name
            );
        peripheral.discover_services().await?;
        let services = peripheral.services();
        let has_service: bool = services.iter().any(|s| s.uuid == *TREADMILL_SERVICE_UUID);
        if is_connected {
            tracing::debug!("Disconnecting from peripheral {:?}", &local_name);
            peripheral
                .disconnect()
                .await
                .expect("Error disconnecting from BLE peripheral");
        }
        if has_service {
            treadmills.push(peripheral)
        }
    }

    Ok(treadmills)
}
