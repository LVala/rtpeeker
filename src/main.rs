#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod rtp_packets_table;
pub mod rtp_sniffer;
pub mod view_state;
use eframe::egui;
use view_state::ViewState;

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
