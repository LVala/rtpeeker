use super::raw::RawPacket;
use rtcp::packet::{self, Packet};

pub struct RtcpPacket<'a> {
    pub raw_packet: &'a RawPacket,
    pub packet: Box<dyn Packet + Send + Sync>,
}

impl<'a> RtcpPacket<'a> {
    pub fn build(packet: &'a RawPacket) -> Option<Vec<RtcpPacket>> {
        let mut buffer: &[u8] = &packet.payload;
        if let Ok(packets) = packet::unmarshal(&mut buffer) {
            let mut rtcp_packets = Vec::new();
            for rtcp_packet in packets {
                rtcp_packets.push(RtcpPacket {
                    raw_packet: packet,
                    packet: rtcp_packet,
                })
            }

            Some(rtcp_packets)
        } else {
            None
        }
    }
}
