pub use pcap::Device;
use pcap::{Activated, Active, Capture, Offline};
use raw::RawPacket;
use std::path::Path;

pub mod raw;
pub mod rtcp;
pub mod rtp;

pub struct Sniffer<T: pcap::State> {
    capture: Capture<T>,
}

impl Sniffer<Offline> {
    pub fn from_file(file: &Path) -> Self {
        let capture = Capture::from_file(file).unwrap();

        Sniffer { capture }
    }
}

impl Sniffer<Active> {
    pub fn from_device(device: Device) -> Self {
        let capture = Capture::from_device(device).unwrap().open().unwrap();
        let capture = capture.setnonblock().unwrap();

        Sniffer { capture }
    }
}

impl<T: Activated> Sniffer<T> {
    pub fn next_packet(&mut self) -> Option<RawPacket> {
        if let Ok(packet) = self.capture.next_packet() {
            RawPacket::build(&packet)
        } else {
            None
        }
    }
}
