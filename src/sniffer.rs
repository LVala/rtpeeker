use futures_util::StreamExt;
use pcap::{Capture, PacketCodec, PacketStream};
use rtpeeker_common::{Packet, Source};

#[derive(Debug)]
pub enum Error {
    CouldntReceivePacket,
    FileNotFound,
    DeviceNotFound,
    DeviceUnavailable,
    UnsupportedPacketType,
    InvalidFilter,
    PacketStreamUnavailable,
}

struct PacketDecoder {
    packet_id: usize,
}

impl PacketDecoder {
    pub fn new() -> Self {
        Self { packet_id: 1 }
    }
}

impl PacketCodec for PacketDecoder {
    type Item = Result<Packet, Error>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        let res = match Packet::build(&packet, self.packet_id) {
            Some(packet) => Ok(packet),
            None => Err(Error::UnsupportedPacketType),
        };

        self.packet_id += 1;
        res
    }
}

// well, it's not technically a Stream...
struct OfflineStream {
    capture: Capture<pcap::Offline>,
    decoder: PacketDecoder,
}

impl OfflineStream {
    pub fn new(capture: Capture<pcap::Offline>, decoder: PacketDecoder) -> Self {
        Self { capture, decoder }
    }

    pub fn next(&mut self) -> Option<Result<Result<Packet, Error>, pcap::Error>> {
        let packet = match self.capture.next_packet() {
            Err(pcap::Error::NoMorePackets) => return None,
            Err(err) => return Some(Err(err)),
            Ok(packet) => packet,
        };

        Some(Ok(self.decoder.decode(packet)))
    }
}

enum CaptureType {
    Offline(OfflineStream),
    Online(PacketStream<pcap::Active, PacketDecoder>),
}

pub struct Sniffer {
    capture: CaptureType,
    pub source: Source,
}

impl Sniffer {
    pub fn from_file(file: &str) -> Result<Self, Error> {
        let Ok(capture) = pcap::Capture::from_file(file) else {
            return Err(Error::FileNotFound);
        };

        let decoder = PacketDecoder::new();
        let stream = OfflineStream::new(capture, decoder);

        Ok(Self {
            capture: CaptureType::Offline(stream),
            source: Source::File(file.to_string()),
        })
    }

    pub fn from_device(device: &str) -> Result<Self, Error> {
        let Ok(capture) = pcap::Capture::from_device(device) else {
            return Err(Error::DeviceNotFound);
        };

        let Ok(capture) = capture.immediate_mode(true).open() else {
            return Err(Error::DeviceUnavailable);
        };

        let Ok(capture) = capture.setnonblock() else {
            return Err(Error::DeviceUnavailable);
        };

        let decoder = PacketDecoder::new();
        let Ok(stream) = capture.stream(decoder) else {
            return Err(Error::PacketStreamUnavailable);
        };

        Ok(Self {
            capture: CaptureType::Online(stream),
            source: Source::Interface(device.to_string()),
        })
    }

    pub fn apply_filter(&mut self, filter: &str) -> Result<(), Error> {
        match self.capture {
            CaptureType::Online(ref mut stream) => stream.capture_mut().filter(filter, true),
            CaptureType::Offline(ref mut stream) => stream.capture.filter(filter, true),
        }
        .map_err(|_| Error::InvalidFilter)
    }

    pub async fn next_packet(&mut self) -> Option<Result<Packet, Error>> {
        let packet = match self.capture {
            CaptureType::Offline(ref mut stream) => stream.next(),
            CaptureType::Online(ref mut stream) => stream.next().await,
        };

        match packet {
            None => None,
            Some(Err(_)) => Some(Err(Error::CouldntReceivePacket)),
            Some(Ok(pack)) => Some(pack),
        }
    }
}
