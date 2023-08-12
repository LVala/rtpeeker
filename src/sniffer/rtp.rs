use rtp::packet::Packet;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc: Vec<u32>,
    pub payload_length: usize, // extension information skipped
}

impl RtpPacket {
    pub fn build(packet: &super::Packet) -> Option<Self> {
        let mut buffer: &[u8] = &packet.payload;
        let Ok(Packet { header, payload }) = Packet::unmarshal(&mut buffer) else {
            return None;
        };

        Some(Self {
            version: header.version,
            padding: header.padding,
            extension: header.extension,
            marker: header.marker,
            payload_type: header.payload_type,
            sequence_number: header.sequence_number,
            timestamp: header.timestamp,
            ssrc: header.ssrc,
            csrc: header.csrc,
            payload_length: payload.len(),
        })
    }
}
