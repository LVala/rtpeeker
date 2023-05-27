use super::raw::{RawPacket, TransportProtocol::Tcp};
use rtp::packet::Packet;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug)]
pub struct RtpPacket {
    pub packet: Packet,
    pub raw_packet: RawPacket,
}

impl RtpPacket {
    pub fn build(packet: RawPacket) -> Option<Self> {
        if let Tcp = packet.transport_protocol {
            return None;
        }
        let mut buffer: &[u8] = &packet.payload;
        if let Ok(rtp_packet) = Packet::unmarshal(&mut buffer) {
            let converted_packet = Self {
                raw_packet: packet,
                packet: rtp_packet,
            };
            Some(converted_packet)
        } else {
            None
        }
    }
}
