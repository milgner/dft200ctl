mod scan;

use std::error::Error;
use std::time::Duration;
use btleplug::api::Peripheral;
use clap::{Parser, Subcommand};

#[macro_use]
extern crate lazy_static;

const DEFAULT_SCAN_DURATION: Duration = Duration::from_secs(2);

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
            scan::find_treadmills(duration)
                .await?
                .into_iter()
                .for_each(|p| println!("Device {:?}", p.id()));
        }
        Commands::SetSpeed { device_address, speed } => {
            println!("Going to set {} to {}", device_address, speed);
        }
    }

    Ok(())
}
