use super::{RtcpPacket, RtpPacket};
use bincode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use etherparse::{
    IpHeader::{self, Version4, Version6},
    Ipv4Header, Ipv6Header, PacketHeaders, TcpHeader,
    TransportHeader::{self, Tcp, Udp},
    UdpHeader,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum SessionProtocol {
    Unknown,
    Rtp,
    Rtcp,
}

impl SessionProtocol {
    pub fn all() -> Vec<Self> {
        vec![Self::Unknown, Self::Rtp, Self::Rtcp]
    }
}

impl fmt::Display for SessionProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Self::Unknown => "Unknown",
            Self::Rtp => "RTP",
            Self::Rtcp => "RTCP",
        };

        write!(f, "{}", res)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Self::Udp => "UDP",
            Self::Tcp => "TCP",
        };

        write!(f, "{}", res)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SessionPacket {
    Unknown,
    Rtp(RtpPacket),
    Rtcp(Vec<RtcpPacket>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    pub payload: Option<Vec<u8>>,
    pub id: usize,
    pub timestamp: Duration,
    pub length: u32,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub transport_protocol: TransportProtocol,
    pub session_protocol: SessionProtocol,
    pub contents: SessionPacket,
}

impl Packet {
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Packet {
    pub fn build(raw_packet: &pcap::Packet, id: usize) -> Option<Self> {
        let Ok(packet) = PacketHeaders::from_ethernet_slice(raw_packet) else {
            return None;
        };
        let PacketHeaders {
            ip: Some(ip),
            transport: Some(transport),
            ..
        } = packet
        else {
            return None;
        };

        let transport_protocol = get_transport_protocol(&transport)?;
        let (source_addr, destination_addr) = convert_addr(&ip, &transport)?;
        let duration = get_duration(raw_packet);

        Some(Self {
            payload: Some(packet.payload.to_vec()),
            id,
            length: raw_packet.header.len,
            timestamp: duration,
            source_addr,
            destination_addr,
            transport_protocol,
            session_protocol: SessionProtocol::Unknown,
            contents: SessionPacket::Unknown,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        // TODO: need a nicer way to temporarily get rid of payload field
        let wo_payload = Self {
            payload: None,
            id: self.id,
            timestamp: self.timestamp,
            length: self.length,
            source_addr: self.source_addr,
            destination_addr: self.destination_addr,
            transport_protocol: self.transport_protocol,
            session_protocol: self.session_protocol,
            contents: self.contents.clone(),
        };
        bincode::serialize(&wo_payload)
    }

    pub fn guess_payload(&mut self) {
        // could use port to determine validity
        // TODO: STUN data, TURN channels, RTCP
        if self.transport_protocol != TransportProtocol::Udp {
            return;
        }

        if let Some(rtp) = RtpPacket::build(self) {
            if is_rtp(&rtp) {
                self.session_protocol = SessionProtocol::Rtp;
                self.contents = SessionPacket::Rtp(rtp);
                return;
            }
        }

        if let Some(rtcp) = RtcpPacket::build(self) {
            if is_rtcp(&rtcp) {
                self.session_protocol = SessionProtocol::Rtcp;
                self.contents = SessionPacket::Rtcp(rtcp);
            }
        }
    }

    pub fn parse_as(&mut self, packet_type: SessionProtocol) {
        if let SessionProtocol::Rtp = packet_type {
            let Some(rtp) = RtpPacket::build(self) else {
                return;
            };
            self.session_protocol = SessionProtocol::Rtp;
            self.contents = SessionPacket::Rtp(rtp);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_rtp(packet: &RtpPacket) -> bool {
    if packet.version != 2 {
        return false;
    }
    if let 72..=76 = packet.payload_type {
        return false;
    }

    true
}

#[cfg(not(target_arch = "wasm32"))]
fn is_rtcp(packets: &[RtcpPacket]) -> bool {
    let Some(first) = packets.first() else {
        return false;
    };

    if !matches!(
        first,
        RtcpPacket::SenderReport(_) | RtcpPacket::ReceiverReport(_)
    ) {
        return false;
    }

    true
}

#[cfg(not(target_arch = "wasm32"))]
fn get_transport_protocol(transport: &TransportHeader) -> Option<TransportProtocol> {
    let protocol = match transport {
        Udp(_) => TransportProtocol::Udp,
        Tcp(_) => TransportProtocol::Tcp,
        _ => return None,
    };

    Some(protocol)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_duration(raw_packet: &pcap::Packet) -> Duration {
    use std::ops::Add;

    // i64 -> u64, but seconds should never be negative
    let secs = raw_packet.header.ts.tv_sec.try_into().unwrap();
    let micrs = raw_packet.header.ts.tv_usec.try_into().unwrap();

    let sec_duration = Duration::from_secs(secs);
    let micros_duration = Duration::from_micros(micrs);

    sec_duration.add(micros_duration)
}

#[cfg(not(target_arch = "wasm32"))]
fn convert_addr(
    ip_header: &IpHeader,
    transport: &TransportHeader,
) -> Option<(SocketAddr, SocketAddr)> {
    use std::net::{
        IpAddr::{V4, V6},
        Ipv4Addr, Ipv6Addr,
    };

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

#[cfg(not(target_arch = "wasm32"))]
fn to_u16(buf: &[u8; 16]) -> Vec<u16> {
    buf.iter()
        .step_by(2)
        .zip(buf.iter().skip(1).step_by(2))
        .map(|(a, b)| ((*a as u16) << 8) | *b as u16)
        .collect::<Vec<_>>()
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn to_u16_works() {
        let init_buf: [u8; 16] = [
            0x15, 0x23, 0x00, 0x11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x88, 0x01,
        ];
        let new_buf = to_u16(&init_buf);
        let valid: [u16; 8] = [0x1523, 0x11, 0, 0, 0, 0, 0, 0x8801];

        assert_eq!(new_buf, valid);
    }
}
