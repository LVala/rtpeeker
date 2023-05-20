use etherparse::{
    IpHeader::{self, Version4, Version6},
    Ipv4Header, PacketHeaders, TcpHeader,
    TransportHeader::{self, Tcp, Udp},
    UdpHeader,
};
pub use pcap::Device;
use pcap::{Activated, Active, Capture, Offline};
use std::net::{IpAddr::V4, Ipv4Addr, SocketAddr};
use std::{path::Path, time::Duration};

#[derive(Debug)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

#[derive(Debug)]
pub struct Packet<'a> {
    pub payload: &'a [u8],
    pub timestamp: Duration,
    pub length: u32,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub transport_protocol: TransportProtocol,
}

impl<'a> Packet<'a> {
    pub fn build(raw_packet: pcap::Packet<'a>) -> Option<Self> {
        let packet = PacketHeaders::from_ethernet_slice(raw_packet.data).unwrap();
        if let PacketHeaders {
            ip: Some(ip_header),
            transport: Some(transport),
            ..
        } = packet
        {
            let transport_protocol = Self::get_transport_protocol(&transport)?;
            let (source_addr, destination_addr) = Self::convert_addr(&ip_header, &transport)?;
            let timestamp = Duration::new(
                raw_packet.header.ts.tv_sec.try_into().unwrap(),
                raw_packet.header.ts.tv_usec.try_into().unwrap(),
            );

            Some(Packet {
                payload: packet.payload,
                length: raw_packet.header.len,
                timestamp,
                source_addr,
                destination_addr,
                transport_protocol,
            })
        } else {
            None
        }
    }

    fn get_transport_protocol(transport: &TransportHeader) -> Option<TransportProtocol> {
        let protocol = match transport {
            Udp(_) => TransportProtocol::UDP,
            Tcp(_) => TransportProtocol::TCP,
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
            Version6(_, _) => panic!("IPv6 currently not supported"),
        };
        let source = SocketAddr::new(source_ip_addr, source_port);
        let destination = SocketAddr::new(dest_ip_addr, dest_port);
        Some((source, destination))
    }
}

pub struct Sniffer<T: pcap::State> {
    capture: Capture<T>,
}

impl Sniffer<Offline> {
    pub fn from_file(file: &Path) -> Self {
        let capture = Capture::from_file(file).unwrap();

        Sniffer { capture }
    }
}

impl Sniffer<Active> {
    pub fn from_device(device: Device) -> Self {
        let capture = Capture::from_device(device).unwrap().open().unwrap();

        Sniffer { capture }
    }
}

impl<T: Activated> Sniffer<T> {
    pub fn next_packet(&mut self) -> Option<Packet> {
        if let Ok(packet) = self.capture.next_packet() {
            Packet::build(packet)
        } else {
            None
        }
    }
}
