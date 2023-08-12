use packet::Packet;
use std::result;

pub mod packet;
pub mod rtcp;
pub mod rtp;

pub enum Error {
    FileNotFound,
    DeviceNotFound,
    DeviceUnavailable,
    CouldntReceivePacket,
    UnsupportedPacketType,
}

type Result<T> = result::Result<T, Error>;

pub struct Sniffer<T: pcap::State> {
    capture: pcap::Capture<T>,
}

impl Sniffer<pcap::Offline> {
    pub fn from_file(file: &str) -> Result<Self> {
        match pcap::Capture::from_file(file) {
            Ok(capture) => Ok(Self { capture }),
            Err(_) => Err(Error::FileNotFound),
        }
    }
}

impl Sniffer<pcap::Active> {
    pub fn from_device(device: &str) -> Result<Self> {
        let Ok(capture) = pcap::Capture::from_device(device) else {
            return Err(Error::DeviceNotFound);
        };

        match capture.open() {
            Ok(capture) => Ok(Self { capture }),
            Err(_) => Err(Error::DeviceUnavailable),
        }
    }
}

impl<T: pcap::Activated> Sniffer<T> {
    pub fn next_packet(&mut self) -> Result<Packet> {
        let Ok(packet) = self.capture.next_packet() else {
            return Err(Error::CouldntReceivePacket);
        };

        match Packet::build(&packet) {
            Some(packet) => Ok(packet),
            None => Err(Error::UnsupportedPacketType),
        }
    }
}
