use crate::sniffer::rtp::{PayloadType, RtpPacket};
use std::ops::Sub;
use std::{net::SocketAddr, time::Duration};

#[derive(Debug)]
pub struct Stream<'a> {
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub ssrc: u32,
    // start time?
    // delta
    pub jitter: f64,
    lost_packets: usize,
    packets: Vec<&'a RtpPacket>,
}

impl<'a> Stream<'a> {
    pub fn new(packet: &'a RtpPacket) -> Self {
        Self {
            source_addr: packet.raw_packet.source_addr.clone(),
            destination_addr: packet.raw_packet.destination_addr.clone(),
            ssrc: packet.packet.header.ssrc,
            jitter: 0.0,
            lost_packets: 0,
            packets: vec![packet],
        }
    }

    pub fn add_packet(&mut self, packet: &'a RtpPacket) {
        self.calculate_jitter(packet);
        self.packets.push(packet);
    }

    pub fn num_of_packets(&self) -> usize {
        self.packets.len()
    }

    pub fn lost_packets_percentage(&self) -> usize {
        // TODO: implement
        self.lost_packets / self.packets.len() * 100
    }

    pub fn payload_type(&self) -> &PayloadType {
        let last_packet = self.packets.last().unwrap();
        &last_packet.payload_type
    }

    pub fn duration(&self) -> Duration {
        // TODO: implement
        Duration::new(0, 0)
    }

    fn calculate_jitter(&mut self, packet: &RtpPacket) {
        let last_packet = self.packets.last().unwrap();
        if let Some(clock_rate) = last_packet.payload_type.clock_rate_in_hz {
            if last_packet.packet.header.payload_type != packet.packet.header.payload_type {
                self.jitter = 0.0;
                return;
            }

            let unit_timestamp = 1.0 / clock_rate as f64;

            let arrival_time_difference_result = packet
                .raw_packet
                .timestamp
                .checked_sub(last_packet.raw_packet.timestamp);

            if let Some(arrival_time_difference) = arrival_time_difference_result {
                let timestamp_difference = packet.packet.header.timestamp as f64 * unit_timestamp
                    - last_packet.packet.header.timestamp as f64 * unit_timestamp;
                let d = arrival_time_difference.as_secs_f64() - timestamp_difference;
                self.jitter = self.jitter + (d - self.jitter) / 16.0;
            }
        }
    }
}

#[derive(Debug)]
pub struct Streams<'a> {
    pub streams: Vec<Stream<'a>>,
}

impl<'a> Streams<'a> {
    pub fn new() -> Self {
        Streams {
            streams: Vec::new(),
        }
    }

    pub fn add_packet(&mut self, packet: &'a RtpPacket) {
        for stream in self.streams.iter_mut() {
            if stream.ssrc == packet.packet.header.ssrc {
                stream.add_packet(packet);
                return;
            }
        }
        let new_stream = Stream::new(packet);
        self.streams.push(new_stream);
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::rtp::Stream;
    use crate::mappers::payload_type_mapper;
    use crate::sniffer::raw::{RawPacket, TransportProtocol};
    use crate::sniffer::rtp::{PayloadType, RtpPacket};
    use bytes::Bytes;
    use eframe::egui::CursorIcon::Default;
    use rtp::header::Header;
    use rtp::packet::Packet;
    use std::io::Empty;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::ops::Sub;
    use std::time::Duration;

    #[test]
    fn initial_jitter_is_0() {
        let packet0 = &create_packet(1240, 348411, 8);

        let stream = Stream::new(packet0);

        let expected_jitter = 0.0;
        assert_eq!(stream.jitter, expected_jitter);
    }

    #[test]
    fn calculates_jitter_on_appending_packets() {
        let packet0 = &create_packet(1240, 348411, 8);
        let packet1 = &create_packet(1400, 418358, 8);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);

        let expected_jitter = 0.0031216875;
        let eps = 0.0000000010;
        assert_eq!((stream.jitter - expected_jitter).abs() < eps, true);
    }

    #[test]
    fn calculates_jitter_on_appending_packets2() {
        let packet0 = &create_packet(1240, 348411, 8);
        let packet1 = &create_packet(1400, 418358, 8);
        let packet2 = &create_packet(1560, 421891, 8);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);
        stream.add_packet(packet2);

        let expected_jitter = 0.00189739453125;
        let eps = 0.0000000010;
        assert_eq!((stream.jitter - expected_jitter).abs() < eps, true);
    }

    #[test]
    fn when_next_packet_has_different_payload_type_then_resets_jitter_to_0() {
        let packet0 = &create_packet(1240, 348411, 8);
        let packet1 = &create_packet(1400, 418358, 2);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);

        let expected_jitter = 0.0;
        assert_eq!(stream.jitter, expected_jitter);
    }

    #[test]
    fn when_next_packet_has_payload_type_with_undefined_clock_rate_then_resets_jitter_to_0() {
        let packet0 = &create_packet(1240, 348411, 8);
        let packet1 = &create_packet(1400, 418358, 19);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);

        let expected_jitter = 0.0;
        assert_eq!(stream.jitter, expected_jitter);
    }

    fn create_packet(rtp_timestamp: u32, timestamp_in_micro: u64, payload_type: u8) -> RtpPacket {
        let raw_packet = RawPacket {
            payload: Vec::new(),
            timestamp: Duration::from_micros(timestamp_in_micro),
            length: 0,
            source_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            destination_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            transport_protocol: TransportProtocol::Tcp,
        };

        let mut rtp_packet = RtpPacket {
            packet: Packet {
                header: Header {
                    version: 1u8,
                    padding: false,
                    extension: false,
                    marker: false,
                    payload_type,
                    sequence_number: 0,
                    timestamp: rtp_timestamp,
                    ssrc: 0,
                    csrc: vec![],
                    extension_profile: 0,
                    extensions: vec![],
                },
                payload: Bytes::from_static(&[]),
            },
            raw_packet,
            payload_type: payload_type_mapper::from(payload_type),
        };
        rtp_packet
    }
}
