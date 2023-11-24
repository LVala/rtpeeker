use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Goodbye {
    pub sources: Vec<u32>,
    pub reason: String,
    pub timestamp: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl Goodbye {
    pub fn new(goodbye: &rtcp::goodbye::Goodbye, packet: &Packet) -> Self {
        let reason = std::str::from_utf8(&goodbye.reason[..])
            .unwrap()
            .to_string();

        Self {
            sources: goodbye.sources.clone(),
            reason,
            timestamp: packet.timestamp,
        }
    }
}
