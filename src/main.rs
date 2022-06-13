mod scan;

use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use bluer::{Address, Uuid};
use clap::{Parser, Subcommand};
use tokio::time::timeout;

#[macro_use]
extern crate lazy_static;

const DEFAULT_SCAN_DURATION: Duration = Duration::from_secs(20*10000);

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for devices
    Scan { seconds: Option<u8> },
    /// Set speed of the treadmill
    SetSpeed {
        device_address: String,
        speed: u8
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Scan { seconds } => {
            let duration = seconds.map_or(DEFAULT_SCAN_DURATION, |i| Duration::from_secs(i as u64));
            let device = timeout(duration,scan::find_treadmill()).await?;
            match device {
                Ok(device) => {
                    let name = device.name().await?.unwrap_or("Unnamed".to_string());
                    println!("Found device: {} ({})", name, device.address());
                }
                Err(err) => {
                    eprintln!("No treadmill device found");
                }
            }
        }
        Commands::SetSpeed { device_address: address_string, speed } => {
            println!("Going to set {} to {}", address_string, speed);
            let addr = Address::from_str(address_string)?;

            let session = bluer::Session::new().await?;
            let adapter = session.default_adapter().await?;
            adapter.set_powered(true).await?;
            let device = adapter.device(addr)?;
            device.connect().await?;

            let service = device.service(14).await?;
            // Handle 0x0011 in the debug output
            //let char10 = service.characteristic(15).await.expect("Char 10 not found");
            // handle 0x0013 in the debug output
            let char13 = service.characteristic(18).await?;

            // this resumes running at the last known speed
            // functional equivalent of the power on button of the remote
            // does not toggle, though; on first invocation, speed is at 1
            char13.write(&[0xf0, 0xc3, 0x03, 0x01, 0x00, 0x00, 0xb7]).await;

            // this is sent like some kind of heartbeat?
            //char13.write(&[0xf0, 0xc3, 0x03, 0x00, 0x00, 0x00, 0xb6]).await;

            tokio::time::sleep(Duration::from_secs(8)).await;
            // println!("Doing sth else");
            // this sets speed to 2, regardless of previous speed! 🥳
            char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x14, 0x00, 0xcd]).await;
            // println!("Once more");
            // tokio::time::sleep(Duration::from_secs(8)).await;
            // char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x14, 0x00, 0xcd]).await;

            // not sure what these do, but it beeps - does not beep if preceded by "heartbeat" sequence above
            //char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x0b, 0x00, 0xc4]).await;
            //char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x12, 0x00, 0xcb]).await;
            //char13.write(&[0xf0, 0xc3, 0x03, 0x00, 0x00, 0x00, 0xb6]).await;
            // char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x11, 0x00, 0xca]).await;
            // char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x10, 0x00, 0xc9]).await;
            // no beep
            // char13.write(&[0xf0, 0xc6, 0x01, 0x01, 0xb8]).await;


            //char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x0c, 0x00, 0xc5]).await;

            //char13.write(&[0xf0, 0xc1, 0x02, 0x00, 0x00, 0xb3]).await;
            // char13.write(&[0xf0, 0xc3, 0x03, 0x00, 0x00, 0x00, 0xb6]).await;

            // START OPTIONAL BLOCK
            // blinking all LEDs if there's nothing else
            //char13.write(&[0xf0, 0xc5, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbf]).await;
            //char13.write(&[0xf0, 0xc3, 0x03, 0x00, 0x00, 0x00, 0xb6]).await;
            // goes into "Knight Rider" LED mode
            //char13.write(&[0xf0, 0xc4, 0x05, 0x00, 0x19, 0x01, 0xaf, 0x4b, 0xcd]).await;
            // not sure
            //char13.write(&[0x02, 0x01, 0x00, 0x15, 0x00, 0x11, 0x00, 0x04, 0x00, 0x52, 0x13, 0x00, 0xf0, 0xc5, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbf]).await;
            // END OPTIONAL BLOCK

            // tokio::time::sleep(Duration::from_secs(5)).await;
            // char13.write(&[0xf0, 0xc3, 0x03, 0x01, 0x00, 0x00, 0xb7]).await;
            // tokio::time::sleep(Duration::from_secs(5)).await;
            // char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x0b, 0x00, 0xc4]).await;
            //char13.write(&[0xf0, 0xc1, 0x02, 0x00, 0x00, 0xb3]).await;
            //char13.write(&[0xf0, 0xc3, 0x03, 0x03, 0x0b, 0x00, 0xc4]).await?;
            //characteristic.write(&[0xf0, 0xc3, 0x03, 0x03, 0x14, 0x00, 0xcd]).await?;
            device.disconnect().await;
        }
    }

    Ok(())
}


