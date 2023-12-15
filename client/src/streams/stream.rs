use crate::utils::ntp_to_f64;
use rtpeeker_common::packet::TransportProtocol;
use rtpeeker_common::rtcp::{source_description::SdesType, SourceDescription};
use rtpeeker_common::{Packet, RtcpPacket, RtpPacket};
use std::cmp::{max, min};
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
    pub ntp_time: Option<u64>,
    pub time_delta: Duration,
    pub jitter: Option<f64>,
    pub prev_lost: bool,
    pub bytes: usize,
    pub bitrate: usize,     // in the last second, kbps
    pub packet_rate: usize, // packets/s
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
    pub max_jitter: f64,
    pub cname: Option<String>,
    bytes: usize,
    sum_jitter: f64,
    jitter_count: usize,
    first_sequence_number: u16,
    last_sequence_number: u16,
    first_time: Duration,
    last_time: Duration,
    // ntp synchronization
    pub ntp_rtp: Option<(u64, u32)>,
    pub estimated_clock_rate: Option<f64>,
}

impl Stream {
    pub fn new(packet: &Packet, rtp: &RtpPacket, default_alias: String) -> Self {
        let rtp_info = RtpInfo {
            packet: rtp.clone(),
            id: packet.id,
            time: packet.timestamp,
            ntp_time: None,
            time_delta: Duration::from_secs(0),
            jitter: Some(0.0),
            prev_lost: false,
            bytes: packet.length as usize,
            bitrate: packet.length as usize * 8,
            packet_rate: 1,
        };

        Self {
            source_addr: packet.source_addr,
            destination_addr: packet.destination_addr,
            protocol: packet.transport_protocol,
            ssrc: rtp.ssrc,
            alias: default_alias,
            rtp_packets: vec![rtp_info],
            rtcp_packets: Vec::new(),
            bytes: packet.length as usize,
            max_jitter: 0.0,
            sum_jitter: 0.0,
            jitter_count: 0,
            cname: None,
            first_sequence_number: rtp.sequence_number,
            last_sequence_number: rtp.sequence_number,
            first_time: packet.timestamp,
            last_time: packet.timestamp,
            ntp_rtp: None,
            estimated_clock_rate: None,
        }
    }

    pub fn get_duration(&self) -> Duration {
        self.last_time.checked_sub(self.first_time).unwrap()
    }

    pub fn get_expected_count(&self) -> usize {
        (self.last_sequence_number + 1 - self.first_sequence_number) as usize
    }

    pub fn get_mean_jitter(&self) -> f64 {
        self.sum_jitter / self.jitter_count as f64
    }

    pub fn get_mean_bitrate(&self) -> f64 {
        let duration = self.get_duration().as_secs_f64();
        self.bytes as f64 * 8.0 / duration
    }

    pub fn get_mean_packet_rate(&self) -> f64 {
        let duration = self.get_duration().as_secs_f64();
        self.rtp_packets.len() as f64 / duration
    }

    pub fn add_rtp_packet(&mut self, packet: &Packet, rtp: &RtpPacket) {
        let rtp_info = RtpInfo {
            packet: rtp.clone(),
            id: packet.id,
            time: packet.timestamp,
            ntp_time: None,
            time_delta: Duration::from_secs(0),
            jitter: None,
            prev_lost: true,
            bytes: packet.length as usize,
            bitrate: 0,
            packet_rate: 0,
        };

        self.update_rtp_parameters(rtp_info);
    }

    pub fn add_rtcp_packet(&mut self, id: usize, timestamp: Duration, packet: &RtcpPacket) {
        match &packet {
            RtcpPacket::SourceDescription(sd) => self.update_sdes_items(sd),
            RtcpPacket::ReceiverReport(_rr) => {}
            RtcpPacket::SenderReport(sr) => {
                // let mut revisit_packets = false;
                if let Some((ntp_time, _rtp_time)) = self.ntp_rtp {
                    // revisit_packets = self.estimated_clock_rate.is_none();
                    // let rtp_diff = sr.rtp_time - rtp_time;
                    let _secs_diff = ntp_to_f64(sr.ntp_time) - ntp_to_f64(ntp_time);
                    // self.estimated_clock_rate = Some(rtp_diff as f64 / secs_diff);
                } else {
                    // revisit_packets = true;
                }
                // self.ntp_rtp = Some((sr.ntp_time, sr.rtp_time));
                // TODO: use the estiamted clock rate to set ntp time in rtp_info
                // TODO: sometimes ntp timestamps are bs
            }
            _ => {}
        }

        let rtcp_info = RtcpInfo::new(packet, id, timestamp);
        self.rtcp_packets.push(rtcp_info);
    }

    fn update_rtp_parameters(&mut self, mut rtp_info: RtpInfo) {
        rtp_info.time_delta = rtp_info.time - self.rtp_packets.last().unwrap().time;

        self.estimate_ntp_time(&mut rtp_info);
        self.update_jitter(&mut rtp_info);
        self.update_rates(&mut rtp_info);

        self.bytes += rtp_info.bytes;

        self.first_time = min(self.first_time, rtp_info.time);
        self.last_time = max(self.last_time, rtp_info.time);
        self.first_sequence_number =
            min(self.first_sequence_number, rtp_info.packet.sequence_number);
        self.last_sequence_number = max(self.last_sequence_number, rtp_info.packet.sequence_number);

        self.update_prev_lost(&mut rtp_info);
        self.rtp_packets.push(rtp_info);
    }

    fn estimate_ntp_time(&self, _rtp_info: &mut RtpInfo) {
        // TODO
    }

    fn update_jitter(&mut self, rtp_info: &mut RtpInfo) {
        let Some(clock_rate) = rtp_info.packet.payload_type.clock_rate else {
            return;
        };

        let prev_rtp_info = self.rtp_packets.last().unwrap();

        let is_new = rtp_info.packet.payload_type.id != prev_rtp_info.packet.payload_type.id;
        if is_new {
            rtp_info.jitter = Some(0.0);
            return;
        }

        let unit = 1.0 / clock_rate as f64;
        let arrival_diff = (rtp_info.time - prev_rtp_info.time).as_secs_f64();
        let rtp_timestamp_diff =
            (rtp_info.packet.timestamp as i64 - prev_rtp_info.packet.timestamp as i64) as f64;
        let diff = arrival_diff - rtp_timestamp_diff * unit;

        let prev_jitter = prev_rtp_info.jitter.unwrap();
        let jitter = prev_jitter + (diff.abs() - prev_jitter) / 16.0;

        rtp_info.jitter = Some(jitter);

        if jitter > self.max_jitter {
            self.max_jitter = jitter;
        }
        self.sum_jitter += jitter;
        self.jitter_count += 1;
    }

    fn update_rates(&self, rtp_info: &mut RtpInfo) {
        let cutoff = rtp_info.time.checked_sub(Duration::from_secs(1)).unwrap();

        let last_second_packets = self.rtp_packets.iter().rev().map_while(|pack| match pack {
            RtpInfo { time, .. } if *time > cutoff => Some(pack.bytes),
            _ => None,
        });

        // remember to include the `rtp_info` packet
        // as it hasn't been placed in `rtp_packets` yet
        rtp_info.packet_rate = last_second_packets.clone().count() + 1;

        let bytes = last_second_packets.sum::<usize>() + rtp_info.bytes;
        rtp_info.bitrate = bytes * 8;
    }

    fn update_prev_lost(&mut self, rtp_info: &mut RtpInfo) {
        if rtp_info.packet.sequence_number == self.first_sequence_number {
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
