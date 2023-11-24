use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationDefined {
    pub timestamp: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl ApplicationDefined {
    pub fn new(packet: &Packet) -> Self {
        Self {
            timestamp: packet.timestamp,
        }
    }
}
