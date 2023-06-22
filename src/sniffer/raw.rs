use crate::sniffer::rtcp::RtcpPacketGroup;
use crate::sniffer::rtp::RtpPacket;
use etherparse::{
    IpHeader::{self, Version4, Version6},
    Ipv4Header, PacketHeaders, TcpHeader,
    TransportHeader::{self, Tcp, Udp},
    UdpHeader,
};
use pcap::Packet;
use std::fmt::{Display, Error, Formatter};
use std::net::{IpAddr::V4, Ipv4Addr, SocketAddr};
use std::ops::Add;
use std::time::Duration;

#[derive(Debug)]
pub enum SessionPacket {
    Unknown,
    RTP(RtpPacket),
    RTCP(RtcpPacketGroup),
}

impl Display for SessionPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let name = match self {
            Self::Unknown => "Unknown",
            Self::RTP(_) => "RTP",
            Self::RTCP(_) => "RTCP",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

impl Display for TransportProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let name = match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PacketTypeId {
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub protocol: TransportProtocol,
}

impl PacketTypeId {
    pub fn new(
        source_addr: SocketAddr,
        destination_addr: SocketAddr,
        protocol: TransportProtocol,
    ) -> Self {
        PacketTypeId {
            source_addr,
            destination_addr,
            protocol,
        }
    }
}

#[derive(Debug)]
pub struct RawPacket {
    pub payload: Vec<u8>,
    pub timestamp: Duration,
    pub length: u32,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub transport_protocol: TransportProtocol,
    pub session_packet: SessionPacket,
}

impl RawPacket {
    pub fn build(raw_packet: &Packet) -> Option<Self> {
        let packet = PacketHeaders::from_ethernet_slice(raw_packet.data).unwrap();
        if let PacketHeaders {
            ip: Some(ip_header),
            transport: Some(transport),
            ..
        } = packet
        {
            let transport_protocol = get_transport_protocol(&transport)?;
            let (source_addr, destination_addr) = convert_addr(&ip_header, &transport)?;

            let sec_duration = Duration::from_secs(raw_packet.header.ts.tv_sec.try_into().unwrap());
            let micros_duration =
                Duration::from_micros(raw_packet.header.ts.tv_usec.try_into().unwrap());

            let duration = sec_duration.add(micros_duration);
            Some(Self {
                payload: packet.payload.to_vec(), // FIXME: borrow checker won this time
                length: raw_packet.header.len,
                timestamp: duration,
                source_addr,
                destination_addr,
                transport_protocol,
                session_packet: SessionPacket::Unknown,
            })
        } else {
            None
        }
    }

    pub fn parse_as_rtp(&mut self) {
        let rtp_packet = RtpPacket::build(self).unwrap();
        self.session_packet = SessionPacket::RTP(rtp_packet);
    }

    pub fn parse_as_rtcp(&mut self) {
        let rtcp_packet_group = RtcpPacketGroup::rtcp_packets_from(self).unwrap();
        self.session_packet = SessionPacket::RTCP(rtcp_packet_group);
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
        // FIXME: support IPv6
        Version6(_, _) => return None,
    };
    let source = SocketAddr::new(source_ip_addr, source_port);
    let destination = SocketAddr::new(dest_ip_addr, dest_port);
    Some((source, destination))
}
