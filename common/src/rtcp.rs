use serde::{Deserialize, Serialize};

mod goodbye;
mod receiver_report;
mod reception_report;
mod sender_report;
mod source_description;

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
            .map(|(packet, packet_type)| Self::cast_to_packet(packet, packet_type))
            .collect();

        Some(packets)
    }

    fn cast_to_packet(
        packet: Box<dyn rtcp::packet::Packet>,
        packet_type: rtcp::header::PacketType,
    ) -> Self {
        use rtcp::header::PacketType;

        match packet_type {
            PacketType::SenderReport => {
                let pack = packet.as_any().downcast_ref().unwrap();
                RtcpPacket::SenderReport(sender_report::SenderReport::new(pack))
            }
            PacketType::ReceiverReport => {
                let pack = packet.as_any().downcast_ref().unwrap();
                RtcpPacket::ReceiverReport(receiver_report::ReceiverReport::new(pack))
            }
            PacketType::SourceDescription => {
                let pack = packet.as_any().downcast_ref().unwrap();
                RtcpPacket::SourceDescription(source_description::SourceDescription::new(pack))
            }
            PacketType::Goodbye => {
                let pack = packet.as_any().downcast_ref().unwrap();
                RtcpPacket::Goodbye(goodbye::Goodbye::new(pack))
            }
            PacketType::ApplicationDefined => RtcpPacket::ApplicationDefined,
            PacketType::PayloadSpecificFeedback => RtcpPacket::PayloadSpecificFeedback,
            PacketType::TransportSpecificFeedback => RtcpPacket::TransportSpecificFeedback,
            PacketType::ExtendedReport => RtcpPacket::ExtendedReport,
            PacketType::Unsupported => RtcpPacket::Other,
        }
    }
}
