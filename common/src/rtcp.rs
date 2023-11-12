pub use goodbye::Goodbye;
pub use receiver_report::ReceiverReport;
pub use reception_report::ReceptionReport;
pub use sender_report::SenderReport;
use serde::{Deserialize, Serialize};
pub use source_description::SourceDescription;

pub mod application_defined;
pub mod extended_report;
pub mod goodbye;
pub mod other;
pub mod payload_specific_feedback;
pub mod receiver_report;
pub mod reception_report;
pub mod sender_report;
pub mod source_description;
pub mod transport_specific_feedback;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RtcpPacket {
    SenderReport(sender_report::SenderReport),
    ReceiverReport(receiver_report::ReceiverReport),
    SourceDescription(source_description::SourceDescription),
    Goodbye(goodbye::Goodbye),
    ApplicationDefined(application_defined::ApplicationDefined),
    PayloadSpecificFeedback(payload_specific_feedback::PayloadSpecificFeedback),
    TransportSpecificFeedback(transport_specific_feedback::TransportSpecificFeedback),
    ExtendedReport(extended_report::ExtendedReport),
    Other(other::Other),
}

impl RtcpPacket {
    pub fn get_type_name(&self) -> &str {
        use RtcpPacket::*;

        match self {
            SenderReport(_) => "Sender Report",
            ReceiverReport(_) => "Receiver Report",
            SourceDescription(_) => "Source Description",
            Goodbye(_) => "Goodbye",
            ApplicationDefined(_) => "Application Defined",
            PayloadSpecificFeedback(_) => "Payload-specific Feedback",
            TransportSpecificFeedback(_) => "Transport-specific Feedback",
            ExtendedReport(_) => "Extended Report",
            Other(_) => "Other",
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
            .map(|(rtcp_packet, packet_type)| {
                Self::cast_to_packet(packet, rtcp_packet, packet_type)
            })
            .collect();

        Some(packets)
    }

    fn cast_to_packet(
        packet: &super::Packet,
        rtcp_packet: Box<dyn rtcp::packet::Packet>,
        packet_type: rtcp::header::PacketType,
    ) -> Self {
        use rtcp::header::PacketType;
        match packet_type {
            PacketType::SenderReport => {
                let sr_packet = rtcp_packet.as_any().downcast_ref().unwrap();
                RtcpPacket::SenderReport(sender_report::SenderReport::new(sr_packet, packet))
            }
            PacketType::ReceiverReport => {
                let receiver_report = rtcp_packet.as_any().downcast_ref().unwrap();
                RtcpPacket::ReceiverReport(receiver_report::ReceiverReport::new(
                    receiver_report,
                    packet,
                ))
            }
            PacketType::SourceDescription => {
                let source_description = rtcp_packet.as_any().downcast_ref().unwrap();
                RtcpPacket::SourceDescription(source_description::SourceDescription::new(
                    source_description,
                    packet,
                ))
            }
            PacketType::Goodbye => {
                let goodbye = rtcp_packet.as_any().downcast_ref().unwrap();
                RtcpPacket::Goodbye(goodbye::Goodbye::new(goodbye, packet))
            }
            PacketType::ApplicationDefined => {
                RtcpPacket::ApplicationDefined(application_defined::ApplicationDefined::new(packet))
            }
            PacketType::PayloadSpecificFeedback => RtcpPacket::PayloadSpecificFeedback(
                payload_specific_feedback::PayloadSpecificFeedback::new(packet),
            ),
            PacketType::TransportSpecificFeedback => RtcpPacket::TransportSpecificFeedback(
                transport_specific_feedback::TransportSpecificFeedback::new(packet),
            ),
            PacketType::ExtendedReport => {
                RtcpPacket::ExtendedReport(extended_report::ExtendedReport::new(packet))
            }
            PacketType::Unsupported => RtcpPacket::Other(other::Other::new(packet)),
        }
    }
}
