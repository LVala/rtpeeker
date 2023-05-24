use crate::rtp_packets_table::RtpPacketsTable;
use eframe::egui;
use eframe::egui::{Context, Ui};
use std::path::Path;
use crate::sniffer::{Sniffer, raw::RawPacket};
use crate::sniffer::rtp::RtpPacket;

pub struct ViewState<'a> {
    rtp_packets_table: RtpPacketsTable<'a>,
    is_rtp_packets_table_visible: bool,
    picked_path: Option<String>,
    packets: Vec<RawPacket>
}

impl Default for ViewState<'_> {
    fn default() -> Self {
        let rtp_packets_table = Vec::<RtpPacket>::new();

        // TODO sensible default

    }
}

impl eframe::App for ViewState<'_> {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu bar").show(ctx, |ui| {

            ui.horizontal(|ui| {
                self.open_pcap_file_button(ui);
                self.show_rtp_packets_button(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |_| {
            self.show_or_hide_rtp_packets_window(ctx);
        });
    }
}

impl ViewState<'_> {
    fn new(&mut self) {
        // TODO instead of this function, fill up the self.packets when 
        // path is passed
        if let Some(path) = self.picked_path {
            let mut sniffer = Sniffer::from_file(Path::new(&path));

            while let Some(packet) = sniffer.next_packet() {
                self.packets.push(packet);
            }
        } 
    }

    fn open_pcap_file_button(&mut self, ui: &mut Ui) {
        if ui.button("Open pcap file").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_path = Some(path.display().to_string());
            }
        }
    }

    fn show_rtp_packets_button(&mut self, ui: &mut Ui) {
        let table_button_text = if self.is_rtp_packets_table_visible {
            "Hide RTP packets"
        } else {
            "Show RTP packets"
        };
        if ui.button(table_button_text).clicked() {
            self.is_rtp_packets_table_visible = !self.is_rtp_packets_table_visible
        }
    }

    fn show_or_hide_rtp_packets_window(&mut self, ctx: &Context) {
        if self.is_rtp_packets_table_visible {
            self.rtp_packets_table.show(
                ctx,
                &mut self.is_rtp_packets_table_visible,
                &mut self.picked_path,
            )
        }
    }
}
