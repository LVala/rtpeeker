use bytes::Bytes;
use etherparse::{
    IpHeader::{self, Version4, Version6},
    Ipv4Header, PacketHeaders,
    TransportHeader::Udp,
    UdpHeader,
};
use pcap::Capture;
use rtp::{header::Header, packet::Packet};
use std::net::{IpAddr::V4, Ipv4Addr, SocketAddr};
use std::path::Path;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug)]
pub struct RtpPacket {
    pub destination_addr: SocketAddr,
    pub source_addr: SocketAddr,
    pub rtp_header: Header,
    pub payload: Bytes,
}

impl RtpPacket {
    pub fn build(mut packet_headers: PacketHeaders) -> Option<Self> {
        match packet_headers {
            // FIXME: RTP can be also transported via TCP
            PacketHeaders {
                ip: Some(ip_header),
                transport: Some(Udp(udp_header)),
                ..
            } => {
                // FIXME: this normally might not be an RTP packet
                if let Ok(rtp_packet) = Packet::unmarshal(&mut packet_headers.payload) {
                    let (source_addr, destination_addr) = Self::convert_addr(ip_header, udp_header);
                    let converted_packet = Self {
                        source_addr,
                        destination_addr,
                        rtp_header: rtp_packet.header,
                        payload: rtp_packet.payload,
                    };
                    Some(converted_packet)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn convert_addr(ip_header: IpHeader, udp_header: UdpHeader) -> (SocketAddr, SocketAddr) {
        let (source_ip_addr, dest_ip_addr) = match ip_header {
            Version4(
                Ipv4Header {
                    source: [s0, s1, s2, s3],
                    destination: [d0, d1, d2, d3],
                    ..
                },
                _,
            ) => {
                let source = V4(Ipv4Addr::new(s0, s1, s2, s3));
                let destination = V4(Ipv4Addr::new(d0, d1, d2, d3));
                (source, destination)
            }
            // FIXME: support IPv6
            Version6(_, _) => panic!("IPv6 currently not supported"),
        };
        let source = SocketAddr::new(source_ip_addr, udp_header.source_port);
        let destination = SocketAddr::new(dest_ip_addr, udp_header.destination_port);
        (source, destination)
    }
}

pub fn rtp_from_file(file_name: &Path) -> Vec<RtpPacket> {
    // FIXME: get rid of unwraps
    let mut cap = Capture::from_file(file_name).unwrap();
    let mut packets: Vec<RtpPacket> = Vec::new();

    while let Ok(raw_packet) = cap.next_packet() {
        let packet = PacketHeaders::from_ethernet_slice(raw_packet.data).unwrap();
        if let Some(rtp_packet) = RtpPacket::build(packet) {
            packets.push(rtp_packet)
        }
    }

    packets
}
