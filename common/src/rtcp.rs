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
            .filter_map(|packet| Self::cast_to_packet(packet))
            .collect();

        Some(packets)
    }

    fn cast_to_packet(packet: Box<dyn rtcp::packet::Packet>) -> Option<Self> {
        use rtcp::goodbye::Goodbye;
        use rtcp::receiver_report::ReceiverReport;
        use rtcp::sender_report::SenderReport;
        use rtcp::source_description::SourceDescription;

        let packet = packet.as_any();

        if let Some(pack) = packet.downcast_ref::<SenderReport>() {
            return Some(RtcpPacket::SenderReport(sender_report::SenderReport::new(
                pack,
            )));
        }
        if let Some(pack) = packet.downcast_ref::<ReceiverReport>() {
            return Some(RtcpPacket::ReceiverReport(
                receiver_report::ReceiverReport::new(pack),
            ));
        }
        if let Some(pack) = packet.downcast_ref::<SourceDescription>() {
            return Some(RtcpPacket::SourceDescription(
                source_description::SourceDescription::new(pack),
            ));
        }
        if let Some(pack) = packet.downcast_ref::<Goodbye>() {
            return Some(RtcpPacket::Goodbye(goodbye::Goodbye::new(pack)));
        }

        None
    }
}
