use super::{rtcp::RtcpPacket, rtp::RtpPacket};
use etherparse::{
    IpHeader::{self, Version4, Version6},
    Ipv4Header, Ipv6Header, PacketHeaders, TcpHeader,
    TransportHeader::{self, Tcp, Udp},
    UdpHeader,
};
use std::net::{
    IpAddr::{V4, V6},
    Ipv4Addr, Ipv6Addr, SocketAddr,
};
use std::ops::Add;
use std::time::Duration;

#[derive(Debug)]
pub enum PacketType {
    RtpOverUdp,
    RtcpOverUdp,
}

#[derive(Debug)]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub enum SessionPacket {
    Unknown,
    Rtp(RtpPacket),
    Rtcp(Vec<RtcpPacket>),
}

#[derive(Debug)]
pub struct Packet {
    pub payload: Vec<u8>,
    pub timestamp: Duration,
    pub length: u32,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub transport_protocol: TransportProtocol,
    pub contents: SessionPacket,
}

impl Packet {
    pub fn build(raw_packet: &pcap::Packet) -> Option<Self> {
        let Ok(packet) = PacketHeaders::from_ethernet_slice(raw_packet) else {
            return None;
        };
        let PacketHeaders {
            ip: Some(ip),
            transport: Some(transport),
            ..
        } = packet else {
            return None;
        };

        let transport_protocol = get_transport_protocol(&transport)?;
        let (source_addr, destination_addr) = convert_addr(&ip, &transport)?;
        let duration = get_duration(raw_packet);

        Some(Self {
            payload: packet.payload.to_vec(),
            length: raw_packet.header.len,
            timestamp: duration,
            source_addr,
            destination_addr,
            transport_protocol,
            contents: SessionPacket::Unknown,
        })
    }

    pub fn parse_as(&mut self, packet_type: PacketType) {
        if let PacketType::RtpOverUdp = packet_type {
            let Some(rtp) = RtpPacket::build(self) else {
                return;
            };
            self.contents = SessionPacket::Rtp(rtp);
        }
    }
}

fn get_transport_protocol(transport: &TransportHeader) -> Option<TransportProtocol> {
    let protocol = match transport {
        Udp(_) => TransportProtocol::Udp,
        Tcp(_) => TransportProtocol::Tcp,
        _ => return None,
    };

    Some(protocol)
}

fn get_duration(raw_packet: &pcap::Packet) -> Duration {
    // i64 -> u64, but seconds should never be negative
    let secs = raw_packet.header.ts.tv_sec.try_into().unwrap();
    let micrs = raw_packet.header.ts.tv_usec.try_into().unwrap();

    let sec_duration = Duration::from_secs(secs);
    let micros_duration = Duration::from_micros(micrs);

    sec_duration.add(micros_duration)
}

fn convert_addr(
    ip_header: &IpHeader,
    transport: &TransportHeader,
) -> Option<(SocketAddr, SocketAddr)> {
    let (source_port, dest_port) = match *transport {
        Udp(UdpHeader {
            source_port,
            destination_port,
            ..
        })
        | Tcp(TcpHeader {
            source_port,
            destination_port,
            ..
        }) => (source_port, destination_port),
        _ => return None,
    };

    let (source_ip_addr, dest_ip_addr) = match *ip_header {
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
        Version6(
            Ipv6Header {
                source,
                destination,
                ..
            },
            _,
        ) => {
            let s = to_u16(&source);
            let d = to_u16(&destination);

            let source = V6(Ipv6Addr::new(
                s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
            ));
            let destination = V6(Ipv6Addr::new(
                d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7],
            ));
            (source, destination)
        }
    };
    let source = SocketAddr::new(source_ip_addr, source_port);
    let destination = SocketAddr::new(dest_ip_addr, dest_port);
    Some((source, destination))
}

pub fn to_u16(buf: &[u8; 16]) -> [u16; 8] {
    // TODO: tests
    buf.iter()
        .zip(buf.iter().skip(1))
        .map(|(a, b)| ((*a as u16) << 8) | *b as u16)
        .collect::<Vec<_>>()
        .as_slice()
        .try_into()
        .unwrap()
}
