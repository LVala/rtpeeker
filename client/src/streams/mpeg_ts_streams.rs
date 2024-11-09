use log::warn;
use rtpeeker_common::mpegts::aggregator::MpegtsAggregator;
use rtpeeker_common::mpegts::header::PIDTable;
use rtpeeker_common::mpegts::psi::pat::fragmentary_pat::FragmentaryProgramAssociationTable;
use rtpeeker_common::mpegts::psi::psi_buffer::{FragmentaryPsi, PsiBuffer};
use rtpeeker_common::packet::{SessionPacket, TransportProtocol};
use rtpeeker_common::{MpegtsPacket, Packet};
use std::cmp::{max, min};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug)]
pub struct MpegTsInfo {
    pub packet: MpegtsPacket,
    pub id: usize,
    pub time: Duration,
    pub time_delta: Duration,
}

impl MpegTsInfo {
    pub fn new(packet: &MpegtsPacket, id: usize, time: Duration) -> Self {
        Self {
            packet: packet.clone(),
            id,
            time,
            time_delta: Duration::from_secs(0),
        }
    }
}

#[derive(Debug)]
pub struct MpegTsStream {
    pub source_addr: SocketAddr,
    pub destination_addr: SocketAddr,
    pub protocol: TransportProtocol,
    pub alias: String,
    pub mpegts_packets: Vec<MpegTsInfo>,
    pub mpegts_aggregator: MpegtsAggregator,
    first_time: Duration,
    last_time: Duration,
}

impl MpegTsStream {
    pub fn new(packet: &Packet, mpegts: &MpegtsPacket, default_alias: String) -> Self {
        let mut mpegts_aggregator = MpegtsAggregator::new();
        let mpegts_packet = MpegTsInfo {
            packet: mpegts.clone(),
            id: packet.id,
            time: packet.timestamp,
            time_delta: Duration::from_secs(0),
        };

        if let SessionPacket::Mpegts(mpegts) = packet.clone().contents {
            mpegts.fragments.iter().for_each(|fragment| {
                if let PIDTable::ProgramAssociation = fragment.header.pid {
                    if fragment.payload.is_none() {
                        return;
                    }
                    let payload = fragment.clone().payload.unwrap().data;
                    let pat_fragment = FragmentaryProgramAssociationTable::unmarshall(
                        &*payload,
                        fragment.header.payload_unit_start_indicator,
                    );
                    if pat_fragment.is_none() {
                        return;
                    }
                    let pat_fragment = pat_fragment.unwrap();

                    mpegts_aggregator
                        .pat_buffer
                        .set_last_section_number(pat_fragment.header.last_section_number);

                    mpegts_aggregator.pat_buffer.add_fragment(pat_fragment);
                }
            });
        }

        Self {
            source_addr: packet.source_addr,
            destination_addr: packet.destination_addr,
            protocol: packet.transport_protocol,
            alias: default_alias,
            first_time: packet.timestamp,
            last_time: packet.timestamp,
            mpegts_packets: vec![mpegts_packet],
            mpegts_aggregator,
        }
    }

    pub fn get_duration(&self) -> Duration {
        self.last_time.checked_sub(self.first_time).unwrap()
    }

    pub fn get_mean_packet_rate(&self) -> f64 {
        let duration = self.get_duration().as_secs_f64();
        self.mpegts_packets.len() as f64 / duration
    }

    pub fn add_mpegts_packet(&mut self, packet: &Packet, mpegts: &MpegtsPacket) {
        let mpegts_info = MpegTsInfo {
            packet: mpegts.clone(),
            id: packet.id,
            time: packet.timestamp,
            time_delta: Duration::from_secs(0),
        };

        self.determine_type(packet);

        self.update_mpegts_parameters(mpegts_info);
    }

    fn update_mpegts_parameters(&mut self, mut mpegts_info: MpegTsInfo) {
        mpegts_info.time_delta = mpegts_info
            .time
            .checked_sub(self.mpegts_packets.last().unwrap().time)
            .unwrap_or(Duration::ZERO);

        self.first_time = min(self.first_time, mpegts_info.time);
        self.last_time = max(self.last_time, mpegts_info.time);

        self.mpegts_packets.push(mpegts_info);
    }

    fn recalculate(&mut self) {
        let mut mpegts_packets = std::mem::take(&mut self.mpegts_packets).into_iter();
        let mpegts_info = mpegts_packets.next().unwrap();
        self.first_time = mpegts_info.time;
        self.last_time = mpegts_info.time;
        self.mpegts_packets = vec![mpegts_info];

        mpegts_packets.for_each(|rtp| self.update_mpegts_parameters(rtp));
    }

    fn determine_type(&mut self, packet: &Packet) {
        if let SessionPacket::Mpegts(mpegts) = packet.clone().contents {
            mpegts.fragments.iter().for_each(|fragment| {

                if let PIDTable::ProgramAssociation = fragment.header.pid {
                    if fragment.payload.is_none() {
                        return;
                    }
                    let payload = fragment.clone().payload.unwrap().data;
                    let pat_fragment = FragmentaryProgramAssociationTable::unmarshall(
                        &*payload,
                        fragment.header.payload_unit_start_indicator,
                    );
                    if pat_fragment.is_none() {
                        return;
                    }
                    let pat_fragment = pat_fragment.unwrap();
                    self.mpegts_aggregator
                        .pat_buffer
                        .set_last_section_number(pat_fragment.header.last_section_number);

                    self.mpegts_aggregator.pat_buffer.add_fragment(pat_fragment);
                }
            });
        }
    }
}
