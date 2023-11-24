use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::reception_report::ReceptionReport;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SenderReport {
    pub ssrc: u32,
    pub ntp_time: u64,
    pub rtp_time: u32,
    pub packet_count: u32,
    pub octet_count: u32,
    pub reports: Vec<ReceptionReport>,
    pub timestamp: Duration,
    // ignoring profile extensions ATM
    // as we won't be able to decode them anyways
}

#[cfg(not(target_arch = "wasm32"))]
impl SenderReport {
    pub fn new(sr: &rtcp::sender_report::SenderReport, packet: &Packet) -> Self {
        let reports = sr.reports.iter().map(ReceptionReport::new).collect();

        Self {
            ssrc: sr.ssrc,
            ntp_time: sr.ntp_time,
            rtp_time: sr.rtp_time,
            packet_count: sr.packet_count,
            octet_count: sr.octet_count,
            reports,
            timestamp: packet.timestamp,
        }
    }
}
