#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod rtp_packets_table;
pub mod rtp_sniffer;
pub mod sniffer;
pub mod view_state;
// use eframe::egui;
// use view_state::ViewState;
use sniffer::Sniffer;
use std::path::Path;

// fn main() -> Result<(), eframe::Error> {
//     let options = eframe::NativeOptions {
//         initial_window_size: Some(egui::vec2(1600.0, 640.0)),
//         ..Default::default()
//     };
//     eframe::run_native(
//         "Media Stream Analyzer",
//         options,
//         Box::new(|_cc| Box::<ViewState>::default()),
//     )
// }

fn main() {
    let path = Path::new("./pcap_examples/rtp.pcap");
    let mut sniffer = Sniffer::from_file(path);
    loop {
        if let Some(packet) = sniffer.next_packet() {
            // println!("{:?}", packet);
        }
    }
}
