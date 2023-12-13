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
    pub fn from_file(file: &str, filter: Option<String>) -> Result<Self> {
        let Ok(mut capture) = pcap::Capture::from_file(file) else {
            return Err(Error::FileNotFound);
        };

        if let Some(filter) = filter {
            let Ok(_) = capture.filter(&filter, true) else {
                return Err(Error::InvalidFilter);
            };
        }

        Ok(Self {
            packet_id: 1,
            capture,
            source: Source::File(file.to_string()),
        })
    }
}

impl Sniffer<pcap::Active> {
    pub fn from_device(device: &str, filter: Option<String>) -> Result<Self> {
        let Ok(capture) = pcap::Capture::from_device(device) else {
            return Err(Error::DeviceNotFound);
        };

        let Ok(mut capture) = capture.immediate_mode(true).open() else {
            return Err(Error::DeviceUnavailable);
        };

        if let Some(filter) = filter {
            let Ok(_) = capture.filter(&filter, true) else {
                return Err(Error::InvalidFilter);
            };
        }

        Ok(Self {
            packet_id: 1,
            capture,
            source: Source::Interface(device.to_string()),
        })
    }
}

impl<T: pcap::Activated> Sniffer<T> {
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
