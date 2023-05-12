use etherparse::{IpHeader, PacketHeaders, TransportHeader::Udp, UdpHeader};
use pcap::Capture;
use rtp::packet::Packet;
use std::path::Path;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug)]
pub struct RtpPacket {
    pub ip_header: IpHeader,
    pub udp_header: UdpHeader,
    pub rtp_packet: Packet,
}

pub fn rtp_from_file(file_name: &Path) -> Vec<RtpPacket> {
    let mut cap = Capture::from_file(file_name).unwrap();
    let mut valid_packets: Vec<RtpPacket> = Vec::new();

    while let Ok(raw_packet) = cap.next_packet() {
        let mut packet = PacketHeaders::from_ethernet_slice(raw_packet.data).unwrap();
        if let Some(Udp(udp_header)) = packet.transport {
            if let Ok(rtp_packet) = Packet::unmarshal(&mut packet.payload) {
                let packet = RtpPacket {
                    ip_header: packet.ip.unwrap(),
                    udp_header,
                    rtp_packet,
                };
                valid_packets.push(packet);
            }
        }
    }

    valid_packets
}
