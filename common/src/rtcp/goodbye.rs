use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Goodbye {
    pub sources: Vec<u32>,
    pub reason: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Goodbye {
    pub fn new(packet: &rtcp::goodbye::Goodbye) -> Self {
        let reason = std::str::from_utf8(&packet.reason[..]).unwrap().to_string();

        Self {
            sources: packet.sources.clone(),
            reason,
        }
    }
}
