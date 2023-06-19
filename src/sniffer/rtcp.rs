use super::raw::RawPacket;
use rtcp::packet::{self, Packet};

#[derive(Debug)]
pub struct RtcpPacketGroup {
    pub packets: Vec<Box<dyn Packet + Send + Sync>>,
}

impl RtcpPacketGroup {
    pub fn rtcp_packets_from(packet: &RawPacket) -> Option<RtcpPacketGroup> {
        let mut buffer: &[u8] = &packet.payload;
        if let Ok(rtcp_packets) = packet::unmarshal(&mut buffer) {
            let rtcp_packet_group = Self {
                packets: rtcp_packets,
            };
            Some(rtcp_packet_group)
        } else {
            None
        }
    }
}
