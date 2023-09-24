use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::{Packet, RtpPacket};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug)]
pub struct Stream {
    pub rtp_packets: Vec<usize>,
    // rtcp_packets: Vec<usize>,
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub ssrc: u32,
    pub jitter: f64,
    pub jitter_history: Vec<f64>,
    pub lost_percentage: f64,
    pub duration: Duration,
    pub display_name: String,
    payload_type: u8,
    previous_timestamp: Option<Duration>,
    previous_rtp_timestamp: Option<f64>,
    first_sequence_number: Option<u16>,
    first_timestamp: Option<Duration>,
}

impl Stream {
    pub fn new(
        source_addr: SocketAddr,
        destination_addr: SocketAddr,
        ssrc: u32,
        payload_type: u8,
        display_name: String,
    ) -> Self {
        Self {
            rtp_packets: Vec::new(),
            // rtcp_packets: Vec::new(),
            source_addr,
            destination_addr,
            ssrc,
            jitter: 0.0,
            jitter_history: vec![0.0],
            lost_percentage: 0.0,
            duration: Duration::ZERO,
            display_name,
            payload_type,
            previous_timestamp: None,
            previous_rtp_timestamp: None,
            first_sequence_number: None,
            first_timestamp: None,
        }
    }

    pub fn add_rtp_packet(&mut self, packet: &Packet, _rtp: &RtpPacket) {
        self.rtp_packets.push(packet.id);
        if self.first_timestamp.is_none() {
            self.first_timestamp = Some(packet.timestamp)
        }
        self.calculate_jitter(packet);
        self.calculate_lost_percentage(packet);
        self.calculate_duration(packet);
    }

    fn calculate_lost_percentage(&mut self, packet: &Packet) {
        let SessionPacket::Rtp(ref rtp) = packet.contents else {
            unreachable!();
        };
        let Some(first_sequence_number) = self.first_sequence_number else {
            self.first_sequence_number = Some(rtp.sequence_number);
            return;
        };

        let number_of_packets = self.rtp_packets.len() as f64;
        let last_sequence_number = rtp.sequence_number as f64;
        let expected_number_of_packets = last_sequence_number - first_sequence_number as f64 + 1.0;
        self.lost_percentage = 100.0 - (number_of_packets / expected_number_of_packets) * 100.0;
    }

    fn calculate_jitter(&mut self, packet: &Packet) {
        let SessionPacket::Rtp(ref rtp) = packet.contents else {
            unreachable!();
        };

        let Some(last_timestamp) = self.previous_timestamp else {
            self.previous_timestamp = Some(packet.timestamp);
            self.previous_rtp_timestamp = Some(rtp.timestamp as f64);
            return;
        };

        if rtp.payload_type.clock_rate.is_none() || rtp.payload_type.id != self.payload_type {
            self.payload_type = rtp.payload_type.id;
            self.jitter = 0.0;
            self.jitter_history.clear()
        } else {
            let clock_rate = rtp.payload_type.clock_rate.unwrap();
            let unit_timestamp = 1.0 / clock_rate as f64;
            let arrival_time_difference_result = packet.timestamp.checked_sub(last_timestamp);
            if let Some(arrival_time_difference) = arrival_time_difference_result {
                let timestamp_difference = rtp.timestamp as f64 * unit_timestamp
                    - self.previous_rtp_timestamp.unwrap() * unit_timestamp;
                let d = arrival_time_difference.as_secs_f64() - timestamp_difference;

                self.jitter = self.jitter + (d - self.jitter) / 16.0;
                self.jitter_history.push(self.jitter);
            }
        }

        self.previous_timestamp = Some(packet.timestamp);
        self.previous_rtp_timestamp = Some(rtp.timestamp as f64);
    }

    fn calculate_duration(&mut self, packet: &Packet) {
        self.duration = packet
            .timestamp
            .checked_sub(self.first_timestamp.unwrap())
            .unwrap_or(Duration::ZERO)
    }
}
