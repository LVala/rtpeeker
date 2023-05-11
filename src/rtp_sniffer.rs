use std::path::Path;
use webrtc_util::marshal::Unmarshal;
use etherparse::{PacketHeaders, TransportHeader::Udp};
use rtp::packet::Packet;
use pcap::Capture;

pub fn rtp_from_file(file_name: &Path) {
    let mut cap = Capture::from_file(file_name).unwrap();

    while let Ok(packet) = cap.next_packet() {
        let mut packet_headers = PacketHeaders::from_ethernet_slice(packet.data).unwrap();
        if let Some(Udp(_header)) = packet_headers.transport {
            let rtp_packet = Packet::unmarshal(&mut packet_headers.payload);
            println!("{:?}", rtp_packet);
        }

    }
}
