use super::reception_report::ReceptionReport;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceiverReport {
    pub ssrc: u32,
    pub reports: Vec<ReceptionReport>,
    // ignoring profile extensions ATM
    // as we won't be able to decode them anyways
}

#[cfg(not(target_arch = "wasm32"))]
impl ReceiverReport {
    pub fn new(packet: &rtcp::receiver_report::ReceiverReport) -> Self {
        let reports = packet.reports.iter().map(ReceptionReport::new).collect();

        Self {
            ssrc: packet.ssrc,
            reports,
        }
    }
}
