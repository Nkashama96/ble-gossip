use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::time::{sleep, Duration};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    let adapter = adapters
        .into_iter()
        .next()
        .expect("No BLE adapter found");

    println!("🔍 Starting BLE scan...");
    adapter.start_scan(ScanFilter::default()).await?;

    loop {
        let peripherals = adapter.peripherals().await?;

        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                // manufacturer_data is a HashMap (not Option!)
                for (company_id, payload) in &props.manufacturer_data {
                    println!(
                        "📡 {:?} | Company {} | {:?}",
                        props.address,
                        company_id,
                        payload
                    );
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
