#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod rtp_packets_table;
pub mod sniffer;
pub mod view_state;
use eframe::egui;
use view_state::ViewState;
use sniffer::{rtcp::RtcpPacket, rtp::RtpPacket, Sniffer};
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 640.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Media Stream Analyzer",
        options,
        Box::new(|_cc| Box::<ViewState>::default()),
    )
}

// fn main() {
//     let mut sniffer = Sniffer::from_file(path);
//     let mut packets = Vec::new();
//     let mut rtp_packets = Vec::new();
//
//     while let Some(packet) = sniffer.next_packet() {
//         packets.push(packet);
//     }
//
//     for packet in packets.iter() {
//         if packet.destination_addr.port() == 5001 {
//             RtcpPacket::build(packet);
//         }
//         if let Some(rtp_packet) = RtpPacket::build(packet) {
//             rtp_packets.push(rtp_packet);
//         }
//     }
// }
