#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod egui_traits;
pub mod media_stream_analyzer_view_state;
pub mod rtp_packets_table;
pub mod rtp_sniffer;
use crate::media_stream_analyzer_view_state::MediaStreamAnalyzerViewState;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 640.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Media Stream Analyzer",
        options,
        Box::new(|_cc| Box::<MediaStreamAnalyzerViewState>::default()),
    )
}
