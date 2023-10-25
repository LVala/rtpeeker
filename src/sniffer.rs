use rtpeeker_common::Packet;
use std::result;

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    DeviceNotFound,
    DeviceUnavailable,
    CouldntReceivePacket,
    UnsupportedPacketType,
}

type Result<T> = result::Result<T, Error>;

pub struct Sniffer<T: pcap::State> {
    packet_id: usize,
    capture: pcap::Capture<T>,
    pub source: String,
}

impl Sniffer<pcap::Offline> {
    pub fn from_file(file: &str) -> Result<Self> {
        match pcap::Capture::from_file(file) {
            Ok(capture) => Ok(Self {
                packet_id: 0,
                capture,
                source: file.to_string(),
            }),
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
            Ok(capture) => Ok(Self {
                packet_id: 0,
                capture,
                source: device.to_string(),
            }),
            Err(_) => Err(Error::DeviceUnavailable),
        }
    }
}

impl<T: pcap::Activated> Sniffer<T> {
    pub fn next_packet(&mut self) -> Option<Result<Packet>> {
        let packet = match self.capture.next_packet() {
            Ok(pack) => pack,
            Err(pcap::Error::NoMorePackets) => return None,
            Err(_) => return Some(Err(Error::CouldntReceivePacket)),
        };

        match Packet::build(&packet, self.packet_id) {
            Some(packet) => {
                self.packet_id += 1;
                Some(Ok(packet))
            }
            None => Some(Err(Error::UnsupportedPacketType)),
        }
    }
}
