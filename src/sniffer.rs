use futures_util::StreamExt;
use pcap::{PacketCodec, PacketStream};
use rtpeeker_common::{Packet, Source};
use std::result;

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    DeviceNotFound,
    DeviceUnavailable,
    UnsupportedPacketType,
    InvalidFilter,
    PacketStreamUnavailable,
}

type Result<T> = result::Result<T, Error>;

struct PacketBuilder {
    packet_id: usize,
}

impl PacketBuilder {
    fn new() -> Self {
        Self { packet_id: 1 }
    }
}

impl PacketCodec for PacketBuilder {
    type Item = Result<Packet>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        let res = match Packet::build(&packet, self.packet_id) {
            Some(packet) => Ok(packet),
            None => Err(Error::UnsupportedPacketType),
        };

        self.packet_id += 1;
        res
    }
}

pub struct Sniffer<T: pcap::Activated> {
    stream: PacketStream<T, PacketBuilder>,
    pub source: Source,
}

impl Sniffer<pcap::Offline> {
    pub fn from_file(file: &str) -> Result<Self> {
        let Ok(capture) = pcap::Capture::from_file(file) else {
            return Err(Error::FileNotFound);
        };

        let packet_builder = PacketBuilder::new();
        let Ok(stream) = capture.stream(packet_builder) else {
            return Err(Error::PacketStreamUnavailable);
        };

        Ok(Self {
            stream,
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

        // error
        let Ok(capture) = capture.setnonblock() else {
            return Err(Error::DeviceUnavailable);
        };

        let packet_builder = PacketBuilder::new();
        let Ok(stream) = capture.stream(packet_builder) else {
            return Err(Error::PacketStreamUnavailable);
        };

        Ok(Self {
            stream,
            source: Source::Interface(device.to_string()),
        })
    }
}

impl<T: pcap::Activated> Sniffer<T> {
    pub fn apply_filter(&mut self, filter: &str) -> Result<()> {
        self.stream
            .capture_mut()
            .filter(filter, true)
            .map_err(|_| Error::InvalidFilter)
    }

    pub async fn next_packet(&mut self) -> Option<Result<Packet>> {
        match self.stream.next().await {
            Some(Ok(pack)) => Some(pack),
            Some(Err(_)) => None,
            None => None,
        }
    }
}
