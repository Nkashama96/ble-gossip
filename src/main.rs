use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

const OUR_COMPANY_ID: u16 = 0x1234; // <-- CHANGE THIS
const MAX_CACHE_SIZE: usize = 128;

#[derive(Debug, Clone)]
struct GossipMessage {
    id: u32,
    ttl: u8,
    payload: Vec<u8>,
}

struct GossipCache {
    seen: HashSet<u32>,
}

impl GossipCache {
    fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    fn handle(&mut self, msg: GossipMessage) -> Option<GossipMessage> {
        if self.seen.contains(&msg.id) {
            return None;
        }

        if msg.ttl == 0 {
            return None;
        }

        self.seen.insert(msg.id);

        // Basic size control
        if self.seen.len() > MAX_CACHE_SIZE {
            self.seen.clear();
        }

        Some(GossipMessage {
            ttl: msg.ttl - 1,
            ..msg
        })
    }
}

fn decode_gossip(payload: &[u8]) -> Option<GossipMessage> {
    if payload.len() < 5 {
        return None;
    }

    let id = u32::from_le_bytes(payload[0..4].try_into().ok()?);
    let ttl = payload[4];

    let data = payload[5..].to_vec();

    Some(GossipMessage {
        id,
        ttl,
        payload: data,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().expect("No BLE adapter found");

    println!("🔍 Starting BLE gossip scan...");
    adapter.start_scan(ScanFilter::default()).await?;

    let mut cache = GossipCache::new();

    loop {
        let peripherals = adapter.peripherals().await?;

        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                for (company_id, payload) in &props.manufacturer_data {
                    // STEP 2: Filter our protocol
                    if *company_id != OUR_COMPANY_ID {
                        continue;
                    }

                    // STEP 3: Decode gossip message
                    if let Some(msg) = decode_gossip(payload) {
                        // STEP 4: Dedup + TTL
                        if let Some(updated) = cache.handle(msg) {
                            println!(
                                "🟢 Gossip received from {:?} | id={} ttl={} payload={:?}",
                                props.address,
                                updated.id,
                                updated.ttl,
                                String::from_utf8_lossy(&updated.payload)
                            );

                            // STEP 4 (Simulated): Schedule re-gossip
                            println!(
                                "🔁 (Simulated) Would re-advertise gossip id={} ttl={}",
                                updated.id,
                                updated.ttl
                            );
                        }
                    }
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
