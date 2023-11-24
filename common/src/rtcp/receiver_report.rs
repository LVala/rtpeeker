use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::reception_report::ReceptionReport;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceiverReport {
    pub ssrc: u32,
    pub reports: Vec<ReceptionReport>,
    pub timestamp: Duration,
    // ignoring profile extensions ATM
    // as we won't be able to decode them anyways
}

#[cfg(not(target_arch = "wasm32"))]
impl ReceiverReport {
    pub fn new(receiver_report: &rtcp::receiver_report::ReceiverReport, packet: &Packet) -> Self {
        let reports = receiver_report
            .reports
            .iter()
            .map(ReceptionReport::new)
            .collect();

        Self {
            ssrc: receiver_report.ssrc,
            reports,
            timestamp: packet.timestamp,
        }
    }
}
