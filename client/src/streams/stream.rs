use rtpeeker_common::packet::TransportProtocol;
use rtpeeker_common::rtcp::{source_description::SdesType, SourceDescription};
use rtpeeker_common::{RtcpPacket, RtpPacket};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug)]
pub struct RtcpInfo {
    pub packet: RtcpPacket,
    pub id: usize,
    pub time: Duration,
}

impl RtcpInfo {
    pub fn new(packet: &RtcpPacket, id: usize, time: Duration) -> Self {
        Self {
            packet: packet.clone(),
            id,
            time,
        }
    }
}

#[derive(Debug)]
pub struct RtpInfo {
    pub packet: RtpPacket,
    pub id: usize,
    pub time: Duration,
    pub jitter: Option<f64>,
    pub prev_lost: bool,
    // bitrate the last second, in bits per second
    pub bitrate: usize,
    // per second
    pub packet_rate: usize,
}

impl RtpInfo {
    pub fn new(packet: &RtpPacket, id: usize, time: Duration) -> Self {
        Self {
            packet: packet.clone(),
            id,
            time,
            jitter: None,
            prev_lost: true,
            bitrate: 0,
            packet_rate: 0,
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
    pub rtcp_packets: Vec<RtcpInfo>,
    pub bytes: usize,
    pub lost_percentage: f64,
    pub duration: Duration,
    pub cname: Option<String>,
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
            bytes: 0,
            lost_percentage: 0.0,
            duration: Duration::ZERO,
            cname: None,
            first_sequence_number: None,
            last_sequence_number: None,
            first_time: None,
            last_time: None,
        }
    }

    pub fn add_rtp_packet(&mut self, id: usize, timestamp: Duration, packet: &RtpPacket) {
        let rtp_info = RtpInfo::new(packet, id, timestamp);
        self.update_rtp_parameters(rtp_info);
    }

    pub fn add_rtcp_packet(&mut self, id: usize, timestamp: Duration, packet: &RtcpPacket) {
        match &packet {
            RtcpPacket::SourceDescription(sd) => self.update_sdes_items(sd),
            RtcpPacket::ReceiverReport(_rr) => {}
            RtcpPacket::SenderReport(_sr) => {
                // TODO handle wallclock time etc
                // TODO handle reception reports
            }
            _ => {}
        }

        let rtcp_info = RtcpInfo::new(packet, id, timestamp);
        self.rtcp_packets.push(rtcp_info);
    }

    fn update_rtp_parameters(&mut self, mut rtp_info: RtpInfo) {
        self.update_jitter(&mut rtp_info);
        self.update_rates(&mut rtp_info);

        self.bytes += rtp_info.packet.payload_length;

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

        // update_prev_lost and update_duration requires
        // first_X/last_X updated
        self.update_prev_lost(&mut rtp_info);
        self.update_duration();

        self.rtp_packets.push(rtp_info);

        //  update_lost_percentage requires packet pushed to rtp_packet
        self.update_lost_percentage();
    }

    fn update_jitter(&self, rtp_info: &mut RtpInfo) {
        let Some(clock_rate) = rtp_info.packet.payload_type.clock_rate else {
            return;
        };

        let Some(prev_rtp_info) = self.rtp_packets.last() else {
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
            (rtp_info.packet.timestamp as i64 - prev_rtp_info.packet.timestamp as i64) as f64;
        let diff = arrival_diff - rtp_timestamp_diff * unit;

        let prev_jitter = prev_rtp_info.jitter.unwrap();
        let jitter = prev_jitter + (diff - prev_jitter) / 16.0;

        rtp_info.jitter = Some(jitter);
    }

    fn update_rates(&self, rtp_info: &mut RtpInfo) {
        let cutoff = rtp_info.time.checked_sub(Duration::from_secs(1)).unwrap();

        let last_second_packets = self.rtp_packets.iter().rev().map_while(|pack| match pack {
            RtpInfo { time, .. } if *time > cutoff => Some(pack.packet.payload_length),
            _ => None,
        });

        // remember to include the `rtp_info` packet
        // as it hasn't been placed in `rtp_packets` yet
        rtp_info.packet_rate = last_second_packets.clone().count() + 1;

        let bytes = last_second_packets.sum::<usize>() + rtp_info.packet.payload_length;
        rtp_info.bitrate = bytes * 8;
    }

    fn update_prev_lost(&mut self, rtp_info: &mut RtpInfo) {
        let first_sn = self.first_sequence_number.unwrap();
        if rtp_info.packet.sequence_number == first_sn {
            rtp_info.prev_lost = false;
            return;
        }

        self.rtp_packets
            .iter_mut()
            .rev()
            // FIXME: we only check last 10 packets, may lead to bugs
            .take(10)
            .for_each(|pack| {
                if pack.packet.sequence_number + 1 == rtp_info.packet.sequence_number {
                    rtp_info.prev_lost = false;
                }

                if pack.packet.sequence_number == rtp_info.packet.sequence_number + 1 {
                    pack.prev_lost = false;
                }
            });
    }

    fn update_duration(&mut self) {
        let first_time = self.first_time.unwrap();
        let last_time = self.last_time.unwrap();

        self.duration = last_time.checked_sub(first_time).unwrap();
    }

    fn update_lost_percentage(&mut self) {
        let first_sequence_number = self.first_sequence_number.unwrap();
        let last_sequence_number = self.last_sequence_number.unwrap();

        let expected_count = last_sequence_number - first_sequence_number + 1;

        // FIXME: this somehow overflows
        let lost_count = self.rtp_packets.len() as i64 - expected_count as i64;
        self.lost_percentage = lost_count as f64 / self.rtp_packets.len() as f64 * 100.0;
    }

    fn update_sdes_items(&mut self, source_description: &SourceDescription) {
        // if we added this packet, one of the chunk's sources must be our ssrc
        // thus the unwrap
        let chunk = source_description
            .chunks
            .iter()
            .find(|chunk| chunk.source == self.ssrc)
            .unwrap();

        let cname = chunk
            .items
            .iter()
            .find(|item| item.sdes_type == SdesType::Cname);

        if let Some(cname_val) = cname {
            self.cname = Some(cname_val.text.clone());
        }
    }
}
