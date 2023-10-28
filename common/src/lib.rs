use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::rtcp::RtcpPacket;
pub use crate::rtp::RtpPacket;
pub use packet::Packet;

pub mod packet;
pub mod rtcp;
pub mod rtp;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Source {
    File(String),
    Interface(String),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (icon, name) = match self {
            Self::File(file) => ("📁", file),
            Self::Interface(interface) => ("🌐", interface),
        };

        write!(f, "{} {}", icon, name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    FetchAll,
    Reparse(usize, packet::SessionProtocol),
    ChangeSource(Source),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Packet(Packet),
    Sources(Vec<Source>),
}

impl Request {
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}

impl Response {
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}
