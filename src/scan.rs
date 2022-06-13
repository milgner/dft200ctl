use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use bluer::{AdapterEvent, Address, Device};
use futures::{pin_mut, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::{sleep, timeout},
};
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;

use uuid::Uuid;

lazy_static! {
    static ref TREADMILL_SERVICE_UUID: Uuid = Uuid::parse_str("0000fff0-0000-1000-8000-00805f9b34fb").unwrap();
}

#[derive(Debug)]
pub enum Error {
    BluerError(bluer::Error),
    Unspecific
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Self::Unspecific
    }
}

impl From<bluer::Error> for Error {
    fn from(err: bluer::Error) -> Self {
        Self::BluerError(err)
    }
}


async fn is_treadmill(device: &Device) -> Result<bool, Error> {
    let treadmill = device.uuids().await?
        .unwrap_or_default()
        .into_iter()
        .any(|uuid| uuid.as_u128() == TREADMILL_SERVICE_UUID.as_u128());
    if treadmill {
        print_details(device);
    }
    Ok(treadmill)
}

#[cfg(not(debug_assertions))]
async fn print_details(device: &Device) -> Result<(), Error> {
    Ok(())
}

// Service: 00001801-0000-1000-8000-00805f9b34fb (8)
// Service: 0000180a-0000-1000-8000-00805f9b34fb (9)
// Characteristics: 00002a29-0000-1000-8000-00805f9b34fb (10)
// Characteristics: 00002a23-0000-1000-8000-00805f9b34fb (12)
// Service: 0000fff0-0000-1000-8000-00805f9b34fb (14)
// Characteristics: 0000fff2-0000-1000-8000-00805f9b34fb (18)
// Characteristics: 0000fff1-0000-1000-8000-00805f9b34fb (15)
// Descriptor: Descriptor { adapter_name: hci0, device_address: DF:D6:EB:3F:2D:D0, service_id: 14, characteristic_id: 15, id: 17 }
// Property: Uuid(00002902-0000-1000-8000-00805f9b34fb)
// Property: CachedValue([])
#[cfg(debug_assertions)]
async fn print_details(device: &Device) -> Result<(), Error> {
    device.connect().await?;
    for service in device.services().await? {
        println!("Service: {} ({})", service.uuid().await?, service.id());
        let characteristics = service.characteristics().await.unwrap_or_default();
        for characteristic in characteristics {
            println!("Characteristics: {:?} ({})", characteristic.uuid().await?, characteristic.id());
            for descriptor in characteristic.descriptors().await? {
                println!("Descriptor: {:?}", descriptor);
                for prop in descriptor.all_properties().await? {
                    println!("Property: {:?}", prop);
                }
            }
        }
    }
    Ok(())
}

// Scan for new device. Every new device will be checked whether it is a treadmill.
// It is assumed that one will stay in proximity to the treadmill and as such,
// `DeviceRemoved` events will be ignored.
// If a device is indeed removed (i.e. unplugged or otherwise becomes unavailable),
// downstream code will detect connection issues and deal with it.
pub async fn find_treadmill() -> Result<Device, Error> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    tracing::info!("Discovering devices using Bluetooth adapter {}\n", adapter.name());
    adapter.set_powered(true).await?;

    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);

    while let Some(device_event) = device_events.next().await {
        match device_event {
            AdapterEvent::DeviceAdded(addr) => {
                tracing::info!("Device added: {}", addr);
                let device = adapter.device(addr)?;
                if is_treadmill(&device).await? {
                    return Ok(device)
                }
            }
            _ => (),
        }
    }
    unreachable!()
}
