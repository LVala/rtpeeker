use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use rtcp::packet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RtcpPacket {}

#[cfg(not(target_arch = "wasm32"))]
impl RtcpPacket {
    pub fn build(packet: &super::Packet) -> Option<Self> {
        // payload field should never be empty
        // except for when encoding the packet
        let mut buffer: &[u8] = packet
            .payload
            .as_ref()
            .expect("Packet's payload field is empty");
        let Ok(_rtcp_packets) = packet::unmarshal(&mut buffer) else {
            return None;
        };

        // TODO proper mapping of different RTCP packet types
        Some(Self {})
    }
}
