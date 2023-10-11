use payload_type::PayloadType;
use serde::{Deserialize, Serialize};

pub mod payload_type;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub marker: bool,
    pub payload_type: PayloadType,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc: Vec<u32>,
    pub payload_length: usize, // extension information skipped
    pub previous_packet_is_lost: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl RtpPacket {
    pub fn build(packet: &super::Packet) -> Option<Self> {
        use rtp::packet::Packet;
        use webrtc_util::marshal::Unmarshal;

        // payload field should never be empty
        // except for when encoding the packet
        let mut buffer: &[u8] = packet
            .payload
            .as_ref()
            .expect("Packet's payload field is empty");
        let Ok(Packet { header, payload }) = Packet::unmarshal(&mut buffer) else {
            return None;
        };

        Some(Self {
            version: header.version,
            padding: header.padding,
            extension: header.extension,
            marker: header.marker,
            payload_type: PayloadType::new(header.payload_type),
            sequence_number: header.sequence_number,
            timestamp: header.timestamp,
            ssrc: header.ssrc,
            csrc: header.csrc,
            payload_length: payload.len(),
            previous_packet_is_lost: false,
        })
    }
}
