use rtcp::packet;

#[derive(Debug)]
pub struct RtcpPacket {}

impl RtcpPacket {
    pub fn build(packet: &super::Packet) -> Option<Self> {
        let mut buffer: &[u8] = &packet.payload;
        let Ok(_rtcp_packets) = packet::unmarshal(&mut buffer) else {
            return None;
        };

        // TODO proper mapping of different RTCP packet types
        Some(Self {})
    }
}
