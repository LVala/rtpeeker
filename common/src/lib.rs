use serde::{Deserialize, Serialize};

pub use crate::rtcp::RtcpPacket;
pub use crate::rtp::RtpPacket;
pub use packet::Packet;

pub mod packet;
pub mod rtcp;
pub mod rtp;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Request {
    FetchAll,
    Reparse(usize, packet::SessionProtocol),
}

impl Request {
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}
