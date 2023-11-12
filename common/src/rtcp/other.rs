use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::Packet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Other {
    pub timestamp: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl Other {
    pub fn new(packet: &Packet) -> Self {
        Self {
            timestamp: packet.timestamp,
        }
    }
}
