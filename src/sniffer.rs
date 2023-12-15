use rtpeeker_common::{Packet, Source};
use std::result;

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    DeviceNotFound,
    DeviceUnavailable,
    CouldntReceivePacket,
    UnsupportedPacketType,
    InvalidFilter,
}

type Result<T> = result::Result<T, Error>;

pub struct Sniffer<T: pcap::State> {
    packet_id: usize,
    capture: pcap::Capture<T>,
    pub source: Source,
}

impl Sniffer<pcap::Offline> {
    pub fn from_file(file: &str) -> Result<Self> {
        let Ok(capture) = pcap::Capture::from_file(file) else {
            return Err(Error::FileNotFound);
        };

        Ok(Self {
            packet_id: 1,
            capture,
            source: Source::File(file.to_string()),
        })
    }
}

impl Sniffer<pcap::Active> {
    pub fn from_device(device: &str) -> Result<Self> {
        let Ok(capture) = pcap::Capture::from_device(device) else {
            return Err(Error::DeviceNotFound);
        };

        let Ok(capture) = capture.immediate_mode(true).open() else {
            return Err(Error::DeviceUnavailable);
        };

        Ok(Self {
            packet_id: 1,
            capture,
            source: Source::Interface(device.to_string()),
        })
    }
}

impl<T: pcap::Activated> Sniffer<T> {
    pub fn apply_filter(&mut self, filter: &str) -> Result<()> {
        self.capture
            .filter(filter, true)
            .map_err(|_| Error::InvalidFilter)
    }

    pub fn next_packet(&mut self) -> Option<Result<Packet>> {
        let packet = match self.capture.next_packet() {
            Ok(pack) => pack,
            Err(pcap::Error::NoMorePackets) => return None,
            Err(_) => return Some(Err(Error::CouldntReceivePacket)),
        };

        let res = match Packet::build(&packet, self.packet_id) {
            Some(packet) => Ok(packet),
            None => Err(Error::UnsupportedPacketType),
        };

        self.packet_id += 1;
        Some(res)
    }
}
