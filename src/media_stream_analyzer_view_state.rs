use crate::egui_traits::Window;
use crate::rtp_packets_table;
use eframe::egui;

#[derive(Default)]
pub struct MediaStreamAnalyzerViewState {
    rtp_packets_table: rtp_packets_table::RtpPacketsTable,
    is_rtp_packets_table_visible: bool,
}

impl eframe::App for MediaStreamAnalyzerViewState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu bar").show(ctx, |ui| {
            let table_button_text = if self.is_rtp_packets_table_visible {
                "Hide RTP packets"
            } else {
                "Show RTP packets"
            };
            if ui.button(table_button_text).clicked() {
                self.is_rtp_packets_table_visible = !self.is_rtp_packets_table_visible
            }
        });

        egui::CentralPanel::default().show(ctx, |_| {
            if self.is_rtp_packets_table_visible {
                self.rtp_packets_table
                    .show(ctx, &mut self.is_rtp_packets_table_visible)
            }
        });
    }
}
