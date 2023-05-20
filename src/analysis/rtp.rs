use crate::rtp_sniffer::RtpPacket;
use std::{net::SocketAddr, time::Duration};

#[derive(Debug)]
pub struct Stream<'a> {
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub ssrc: u32,
    // start time?
    pub duration: Duration,
    // delta, jitter
    pub jitter: f64,
    lost_packets: usize,
    packets: Vec<&'a RtpPacket>,
}

impl<'a> Stream<'a> {
    pub fn new(packet: &'a RtpPacket) -> Self {
        Self {
            source_addr: packet.source_addr.clone(),
            destination_addr: packet.destination_addr.clone(),
            ssrc: packet.rtp_header.ssrc,
            duration: Duration::new(0, 0),
            jitter: 0.0,
            lost_packets: 0,
            packets: vec![packet],
        }
    }

    pub fn add_packet(&mut self, packet: &'a RtpPacket) {
        self.calculate_jitter(packet);
        self.packets.push(packet);
    }

    fn calculate_jitter(&mut self, packet: &RtpPacket) {
        let last_packet = self.packets.last().unwrap();
        if let Some(clock_rate) = last_packet.payload_type.clock_rate_in_hz {
            if last_packet.rtp_header.payload_type != packet.rtp_header.payload_type {
                self.jitter = 0.0;
                return;
            }

            let unit_timestamp = 1.0 / clock_rate as f64;
            let arrival_time_difference = packet.arrival_time - last_packet.arrival_time;
            let timestamp_difference = packet.rtp_header.timestamp as f64 * unit_timestamp
                - last_packet.rtp_header.timestamp as f64 * unit_timestamp;
            let d = arrival_time_difference - timestamp_difference;
            self.jitter = self.jitter + (d - self.jitter) / 16.0;
        }
    }

    pub fn num_of_packets(&self) -> usize {
        self.packets.len()
    }

    pub fn lost_packets_percentage(&self) -> usize {
        self.lost_packets / self.packets.len()
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
            if stream.ssrc == packet.rtp_header.ssrc {
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
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use crate::analysis::rtp::{Stream};
    use crate::mappers::payload_mapper;
    use crate::rtp_sniffer::RtpPacket;


    #[test]
    fn initial_jitter_is_0() {
        let packet0 = &create_packet(1240, 0.348411000, 8);

        let stream = Stream::new(packet0);

        let expected_jitter = 0.0;
        assert_eq!(stream.jitter, expected_jitter);
    }

    #[test]
    fn calculates_jitter_on_appending_packets() {
        let packet0 = &create_packet(1240, 0.348411000, 8);
        let packet1 = &create_packet(1400, 0.418358000, 8);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);

        let expected_jitter = 0.0031216875;
        let eps = 0.0000000010;
        assert_eq!((stream.jitter - expected_jitter).abs() < eps, true);
    }


    #[test]
    fn calculates_jitter_on_appending_packets2() {
        let packet0 = &create_packet(1240, 0.348411000, 8);
        let packet1 = &create_packet(1400, 0.418358000, 8);
        let packet2 = &create_packet(1560, 0.421891000, 8);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);
        stream.add_packet(packet2);

        let expected_jitter = 0.00189739453125;
        let eps = 0.0000000010;
        assert_eq!((stream.jitter - expected_jitter).abs() < eps, true);
    }


    #[test]
    fn when_next_packet_has_different_payload_type_then_resets_jitter_to_0() {
        let packet0 = &create_packet(1240, 0.348411000, 8);
        let packet1 = &create_packet(1400, 0.418358000, 2);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);

        let expected_jitter = 0.0;
        assert_eq!(stream.jitter, expected_jitter);
    }

    #[test]
    fn when_next_packet_has_payload_type_with_undefined_clock_rate_then_resets_jitter_to_0() {
        let packet0 = &create_packet(1240, 0.348411000, 8);
        let packet1 = &create_packet(1400, 0.418358000, 19);

        let mut stream = Stream::new(packet0);
        stream.add_packet(packet1);

        let expected_jitter = 0.0;
        assert_eq!(stream.jitter, expected_jitter);
    }

    fn create_packet(timestamp: u32, arrival_time: f64, payload_type: u8) -> RtpPacket {
        let mut packet = RtpPacket {
            destination_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            source_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            rtp_header: Default::default(),
            payload_type: payload_mapper::from(&payload_type) ,
            payload: Default::default(),
            arrival_time,
        };
        packet.rtp_header.timestamp = timestamp;
        packet.rtp_header.payload_type = payload_type;
        packet
    }
}
