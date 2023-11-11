use rtpeeker_common::packet::TransportProtocol;
use rtpeeker_common::RtpPacket;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug)]
pub struct RtpInfo {
    packet: RtpPacket,
    id: usize,
    time: Duration,
    jitter: Option<f64>,
}

impl RtpInfo {
    pub fn new(packet: &RtpPacket, id: usize, time: Duration) -> Self {
        Self {
            packet: packet.clone(),
            id,
            time,
            jitter: None,
        }
    }
}

#[derive(Debug)]
pub struct Stream {
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub protocol: TransportProtocol,
    pub ssrc: u32,
    pub alias: String,
    pub rtp_packets: Vec<RtpInfo>,
    pub rtcp_packets: Vec<usize>,
    pub loss_percentage: f64,
    pub duration: Duration,
    first_sequence_number: Option<u16>,
    last_sequence_number: Option<u16>,
    first_time: Option<Duration>,
    last_time: Option<Duration>,
}

impl Stream {
    pub fn new(
        source_addr: SocketAddr,
        destination_addr: SocketAddr,
        protocol: TransportProtocol,
        ssrc: u32,
        default_alias: String,
    ) -> Self {
        Self {
            source_addr,
            destination_addr,
            protocol,
            ssrc,
            alias: default_alias,
            rtp_packets: Vec::new(),
            rtcp_packets: Vec::new(),
            loss_percentage: 0.0,
            duration: Duration::ZERO,
            first_sequence_number: None,
            last_sequence_number: None,
            first_time: None,
            last_time: None,
        }
    }

    pub fn add_rtp_packet(&mut self, id: usize, timestamp: Duration, packet: &RtpPacket) {
        let rtp_info = RtpInfo::new(&packet, id, timestamp);
        self.rtp_packets.push(rtp_info);
        self.update_parameters();
    }

    // pub fn add_rtcp_packet(&mut self, packet: &Packet) {
    //     self.rtcp_packets.push(packet.id);
    // }

    fn update_parameters(&mut self) {
        let rtp_info = self.rtp_packets.last().unwrap();

        self.first_time = match self.first_time {
            Some(ft) if ft < rtp_info.time => Some(ft),
            _ => Some(rtp_info.time),
        };

        self.last_time = match self.last_time {
            Some(ft) if ft > rtp_info.time => Some(ft),
            _ => Some(rtp_info.time),
        };

        self.first_sequence_number = match self.first_sequence_number {
            Some(fsn) if fsn < rtp_info.packet.sequence_number => Some(fsn),
            _ => Some(rtp_info.packet.sequence_number),
        };

        self.last_sequence_number = match self.last_sequence_number {
            Some(fsn) if fsn > rtp_info.packet.sequence_number => Some(fsn),
            _ => Some(rtp_info.packet.sequence_number),
        };

        self.update_lost_count();
        self.update_duration();
        self.update_jitter();
    }

    fn update_lost_count(&mut self) {
        let first_sequence_number = self.first_sequence_number.unwrap();
        let last_sequence_number = self.last_sequence_number.unwrap();

        let expected_count = last_sequence_number - first_sequence_number;

        let loss_count = self.rtp_packets.len() - expected_count as usize;
        self.loss_percentage = loss_count as f64 / self.rtp_packets.len() as f64 * 100.0;
    }

    fn update_duration(&mut self) {
        let first_time = self.first_time.unwrap();
        let last_time = self.last_time.unwrap();

        self.duration = last_time.checked_sub(first_time).unwrap_or(Duration::ZERO);
    }

    fn update_jitter(&mut self) {
        let rtp_info = self.rtp_packets.last_mut().unwrap();

        let Some(clock_rate) = rtp_info.packet.payload_type.clock_rate else {
            return;
        };

        let len = self.rtp_packets.len();
        let Some(prev_rtp_info) = self.rtp_packets.get(len - 2) else {
            // rtp_info is the first packet
            rtp_info.jitter = Some(0.0);
            return;
        };

        let is_new = rtp_info.packet.payload_type.id != prev_rtp_info.packet.payload_type.id;
        if is_new {
            rtp_info.jitter = Some(0.0);
            return;
        }

        let unit = 1.0 / clock_rate as f64;
        let arrival_diff = rtp_info
            .time
            .checked_sub(prev_rtp_info.time)
            .unwrap()
            .as_secs_f64();
        let rtp_timestamp_diff =
            (rtp_info.packet.timestamp - prev_rtp_info.packet.timestamp) as f64;
        let diff = arrival_diff - rtp_timestamp_diff * unit;

        let prev_jitter = prev_rtp_info.jitter.unwrap();
        let jitter = prev_jitter + (diff - prev_jitter) / 16.0;

        rtp_info.jitter = Some(jitter);
    }
}
