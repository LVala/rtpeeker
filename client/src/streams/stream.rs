use rtpeeker_common::{Packet, RtpPacket};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct Stream {
    pub rtp_packets: Vec<usize>,
    // rtcp_packets: Vec<usize>,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub ssrc: u32,
}

impl Stream {
    pub fn new(source_addr: SocketAddr, destination_addr: SocketAddr, ssrc: u32) -> Self {
        Self {
            rtp_packets: Vec::new(),
            // rtcp_packets: Vec::new(),
            source_addr,
            destination_addr,
            ssrc,
        }
    }

    pub fn add_rtp_packet(&mut self, packet: &Packet, _rtp: &RtpPacket) {
        self.rtp_packets.push(packet.id);
    }
}
