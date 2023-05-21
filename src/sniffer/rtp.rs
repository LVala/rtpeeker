use super::raw::RawPacket;
use rtp::packet::Packet;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug)]
pub struct RtpPacket<'a> {
    pub raw_packet: &'a RawPacket,
    pub packet: Packet,
}

impl<'a> RtpPacket<'a> {
    pub fn build(packet: &'a RawPacket) -> Option<Self> {
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
