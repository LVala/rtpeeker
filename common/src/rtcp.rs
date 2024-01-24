pub use goodbye::Goodbye;
pub use receiver_report::ReceiverReport;
pub use reception_report::ReceptionReport;
pub use sender_report::SenderReport;
use serde::{Deserialize, Serialize};
pub use source_description::SourceDescription;

pub mod goodbye;
pub mod receiver_report;
pub mod reception_report;
pub mod sender_report;
pub mod source_description;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RtcpPacket {
    SenderReport(sender_report::SenderReport),
    ReceiverReport(receiver_report::ReceiverReport),
    SourceDescription(source_description::SourceDescription),
    Goodbye(goodbye::Goodbye),
    ApplicationDefined,
    PayloadSpecificFeedback,
    TransportSpecificFeedback,
    ExtendedReport,
    Other,
}

impl RtcpPacket {
    pub fn get_type_name(&self) -> &str {
        use RtcpPacket::*;

        match self {
            SenderReport(_) => "Sender Report",
            ReceiverReport(_) => "Receiver Report",
            SourceDescription(_) => "Source Description",
            Goodbye(_) => "Goodbye",
            ApplicationDefined => "Application Defined",
            PayloadSpecificFeedback => "Payload-specific Feedback",
            TransportSpecificFeedback => "Transport-specific Feedback",
            ExtendedReport => "Extended Report",
            Other => "Other",
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl RtcpPacket {
    pub fn build(packet: &super::Packet) -> Option<Vec<Self>> {
        use rtcp::packet;

        // payload field should never be empty
        // except for when encoding the packet
        let mut buffer: &[u8] = packet
            .payload
            .as_ref()
            .expect("Packet's payload field is empty");

        let Ok(rtcp_packets) = packet::unmarshal(&mut buffer) else {
            return None;
        };

        let packets: Vec<_> = rtcp_packets
            .into_iter()
            .map(|packet| Self::cast_to_packet(packet))
            .collect();

        Some(packets)
    }

    fn cast_to_packet(packet: Box<dyn rtcp::packet::Packet>) -> Self {
        // previously, I've used the for of rtcp library
        // but for the sake of being able to publish the crate on crates.io
        // I've reverted the changes, so some packets might not be handled properly
        use rtcp::goodbye::Goodbye;
        use rtcp::receiver_report::ReceiverReport;
        use rtcp::sender_report::SenderReport;
        use rtcp::source_description::SourceDescription;

        let packet = packet.as_any();

        if let Some(pack) = packet.downcast_ref::<Goodbye>() {
            return RtcpPacket::Goodbye(goodbye::Goodbye::new(pack));
        }

        if let Some(pack) = packet.downcast_ref::<ReceiverReport>() {
            return RtcpPacket::ReceiverReport(receiver_report::ReceiverReport::new(pack));
        }

        if let Some(pack) = packet.downcast_ref::<SenderReport>() {
            return RtcpPacket::SenderReport(sender_report::SenderReport::new(pack));
        }

        if let Some(pack) = packet.downcast_ref::<SourceDescription>() {
            return RtcpPacket::SourceDescription(source_description::SourceDescription::new(pack));
        }

        RtcpPacket::Other
    }
}
