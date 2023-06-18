use std::fmt::Debug;
use super::raw::{RawPacket, TransportProtocol::Tcp};
use crate::mappers::payload_type_mapper;
use rtp::packet::Packet;
use webrtc_util::marshal::Unmarshal;

#[derive(Debug )]
pub enum MediaType {
    Audio,
    Video,
    AudioVideo,
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
                "Id {} is {}, clock rate is {}hz, type of media: {}",
                self.id, self.name, clock_rate_in_hz, self.media_type
            )
        } else {
            write!(
                fmt,
                "Id {} is {} and it's clock rate is undefined.",
                self.id, self.name
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
            };
            Some(converted_packet)
        } else {
            None
        }
    }
}
