use crate::sniffer::rtp::RtpPacket;
use std::{net::SocketAddr, time::Duration};

#[derive(Debug)]
pub struct Stream<'a> {
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub ssrc: u32,
    // start time?
    pub payload_type: u8,
    // delta, jitter
    lost_packets: usize,
    packets: Vec<&'a RtpPacket>,
}

impl<'a> Stream<'a> {
    pub fn new(packet: &'a RtpPacket) -> Self {
        Self {
            source_addr: packet.raw_packet.source_addr.clone(),
            destination_addr: packet.raw_packet.destination_addr.clone(),
            ssrc: packet.packet.header.ssrc,
            payload_type: packet.packet.header.payload_type,
            lost_packets: 0,
            packets: vec![packet],
        }
    }

    pub fn add_packet(&mut self, packet: &'a RtpPacket) {
        self.packets.push(packet);
    }

    pub fn num_of_packets(&self) -> usize {
        self.packets.len()
    }

    pub fn lost_packets_percentage(&self) -> usize {
        // TODO: implement
        self.lost_packets / self.packets.len()
    }

    pub fn jitter(&self) -> f64 {
        // TODO: implement
        69.420
    }

    pub fn duration(&self) -> Duration {
        Duration::new(0, 0)
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
