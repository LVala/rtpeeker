use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceptionReport {
    pub ssrc: u32,
    pub fraction_lost: u8,
    pub total_lost: u32,
    pub last_sequence_number: u32,
    pub jitter: u32,
    pub last_sender_report: u32,
    pub delay: u32,
}

#[cfg(not(target_arch = "wasm32"))]
impl ReceptionReport {
    pub fn new(report: &rtcp::reception_report::ReceptionReport) -> Self {
        Self {
            ssrc: report.ssrc,
            fraction_lost: report.fraction_lost,
            total_lost: report.total_lost,
            last_sequence_number: report.last_sequence_number,
            jitter: report.jitter,
            last_sender_report: report.last_sender_report,
            delay: report.delay,
        }
    }
}
