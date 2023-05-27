use etherparse::{
    IpHeader::{self, Version4, Version6},
    Ipv4Header, PacketHeaders, TcpHeader,
    TransportHeader::{self, Tcp, Udp},
    UdpHeader,
};
use std::net::{IpAddr::V4, Ipv4Addr, SocketAddr};
use std::ops::Add;
use std::time::Duration;

#[derive(Debug)]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub struct RawPacket {
    pub payload: Vec<u8>,
    pub timestamp: Duration,
    pub length: u32,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub transport_protocol: TransportProtocol,
}

impl RawPacket {
    pub fn build(raw_packet: &pcap::Packet) -> Option<Self> {
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
            let micros_duration = Duration::from_micros(raw_packet.header.ts.tv_usec.try_into().unwrap());

            let duration = sec_duration.add(micros_duration);
            Some(Self {
                payload: packet.payload.to_vec(), // FIXME: borrow checker won this time
                length: raw_packet.header.len,
                timestamp: duration,
                source_addr,
                destination_addr,
                transport_protocol,
            })
        } else {
            None
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
