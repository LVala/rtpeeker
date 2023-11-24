use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransportSpecificFeedback {
    pub timestamp: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl TransportSpecificFeedback {
    pub fn new(packet: &Packet) -> Self {
        Self {
            timestamp: packet.timestamp,
        }
    }
}
