use super::rtp_packets_table::RtpPacketsTable;
use super::streams_table::StreamsTable;
use crate::sniffer::rtp::RtpPacket;
use crate::sniffer::Sniffer;
use eframe::egui;
use eframe::egui::{Context, Ui};
use std::path::Path;

pub struct ViewState {
    is_rtp_packets_table_visible: bool,
    is_streams_table_visible: bool,
    rtp_packets: Vec<RtpPacket>,
}

impl eframe::App for ViewState {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.open_pcap_file_button(ui);
                self.show_rtp_packets_button(ui);
                self.show_streams_button(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |_| {
            self.show_or_hide_rtp_packets_window(ctx);
        });

        egui::CentralPanel::default().show(ctx, |_| {
            self.show_or_hide_streams_window(ctx);
        });
    }
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            is_rtp_packets_table_visible: false,
            is_streams_table_visible: false,
            rtp_packets: Vec::new(),
        }
    }

    fn open_pcap_file_button(&mut self, ui: &mut Ui) {
        if ui.button("Open pcap file").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                let path = path.display().to_string();
                let path = Path::new(&path);
                let mut sniffer = Sniffer::from_file(path);
                while let Some(packet) = sniffer.next_packet() {
                    if let Some(rtp_packet) = RtpPacket::build(packet) {
                        self.rtp_packets.push(rtp_packet);
                    }
                }
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

    fn show_streams_button(&mut self, ui: &mut Ui) {
        let table_button_text = if self.is_streams_table_visible {
            "Hide streams"
        } else {
            "Show streams"
        };
        if ui.button(table_button_text).clicked() {
            self.is_streams_table_visible = !self.is_streams_table_visible
        }
    }

    fn show_or_hide_rtp_packets_window(&mut self, ctx: &Context) {
        if self.is_rtp_packets_table_visible {
            let mut rtp_packets_table = RtpPacketsTable::new(&mut self.rtp_packets);
            rtp_packets_table.show(ctx, self.is_rtp_packets_table_visible);
        }
    }

    fn show_or_hide_streams_window(&mut self, ctx: &Context) {
        if self.is_streams_table_visible {
            let mut streams_table = StreamsTable::new(&self.rtp_packets);
            streams_table.show(ctx, self.is_streams_table_visible);
        }
    }
}
