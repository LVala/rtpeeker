use super::raw::{RawPacket, TransportProtocol::Tcp};
use crate::mappers::payload_type_mapper;
use rtp::packet::Packet;
use std::fmt::Debug;
use std::time::Duration;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug)]
pub enum MediaType {
    Audio,
    Video,
    AudioOrVideo,
}

#[derive(Debug)]
pub struct PayloadType {
    pub id: u8,
    pub name: String,
    pub clock_rate_in_hz: Option<u32>,
    pub media_type: MediaType,
}

impl std::fmt::Display for PayloadType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(clock_rate_in_hz) = self.clock_rate_in_hz {
            write!(
                fmt,
                "Payload type:\n  id = {},\n  name = {},\n  type = {}\n  clock rate = {}hz\n",
                self.id, self.name, self.media_type, clock_rate_in_hz
            )
        } else {
            write!(
                fmt,
                "Payload type:\n  id = {},\n  name = {},\n  type = {}\n  clock rate = undefined\n",
                self.id, self.name, self.media_type
            )
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct RtpPacket {
    pub packet: Packet,
    pub payload_type: PayloadType,
    pub raw_packet_timestamp: Duration,
}

impl RtpPacket {
    pub fn build(packet: &RawPacket) -> Option<Self> {
        if let Tcp = packet.transport_protocol {
            return None;
        }
        let mut buffer: &[u8] = &packet.payload;
        if let Ok(rtp_packet) = Packet::unmarshal(&mut buffer) {
            let converted_packet = Self {
                payload_type: payload_type_mapper::from(rtp_packet.header.payload_type),
                packet: rtp_packet,
                raw_packet_timestamp: packet.timestamp,
            };
            Some(converted_packet)
        } else {
            None
        }
    }
}
