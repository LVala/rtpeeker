use super::packets_table::PacketsTable;
use super::rtp_packets_table::RtpPacketsTable;
use super::streams_table::StreamsTable;
use crate::sniffer::raw::{PacketTypeId, RawPacket};
use crate::sniffer::{Device, Sniffer};
use eframe::egui;
use eframe::egui::{Context, Ui};
use pcap::Active;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

pub struct ViewState {
    is_rtp_packets_table_visible: bool,
    is_streams_table_visible: bool,
    is_packets_table_visible: bool,
    packets: Vec<RawPacket>,
    sniffer: Option<Sniffer<Active>>,
    is_jitter_visible: HashMap<usize, bool>,
    rtp_packet_ids: HashSet<PacketTypeId>,
    rtcp_packet_ids: HashSet<PacketTypeId>,
}

impl eframe::App for ViewState {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(sniffer) = &mut self.sniffer {
            while let Some(mut packet) = sniffer.next_packet() {
                let packet_type_id = PacketTypeId::new(
                    packet.source_addr,
                    packet.destination_addr,
                    packet.transport_protocol,
                );
                if self.rtp_packet_ids.contains(&packet_type_id) {
                    packet.parse_as_rtp();
                } else if self.rtcp_packet_ids.contains(&packet_type_id) {
                    packet.parse_as_rtcp();
                }
                self.packets.push(packet);
            }
        }

        egui::TopBottomPanel::top("menu bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.open_pcap_file_button(ui);
                self.get_live_packets_button(ui);
                self.show_packets_button(ui);
                self.show_rtp_packets_button(ui);
                self.show_streams_button(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |_| {
            self.show_or_hide_packets_window(ctx);
            self.show_or_hide_rtp_packets_window(ctx);
            self.show_or_hide_streams_window(ctx);
        });
    }
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            is_rtp_packets_table_visible: false,
            is_streams_table_visible: false,
            is_packets_table_visible: false,
            packets: Vec::new(),
            sniffer: None,
            rtp_packet_ids: HashSet::new(),
            rtcp_packet_ids: HashSet::new(),
            is_jitter_visible: HashMap::default(),
        }
    }

    fn open_pcap_file_button(&mut self, ui: &mut Ui) {
        if ui.button("Open pcap file").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.packets.clear();
                self.sniffer = None;
                let path = path.display().to_string();
                let path = Path::new(&path);
                let mut sniffer = Sniffer::from_file(path);
                while let Some(mut packet) = sniffer.next_packet() {
                    let packet_type_id = PacketTypeId::new(
                        packet.source_addr,
                        packet.destination_addr,
                        packet.transport_protocol,
                    );
                    if self.rtp_packet_ids.contains(&packet_type_id) {
                        packet.parse_as_rtp();
                    } else if self.rtcp_packet_ids.contains(&packet_type_id) {
                        packet.parse_as_rtcp();
                    }
                    self.packets.push(packet);
                }
            }
        }
    }

    fn get_live_packets_button(&mut self, ui: &mut Ui) {
        if ui.button("Live packets").clicked() {
            let main_device = Device::lookup().unwrap().unwrap();
            let sniffer = Sniffer::from_device(main_device);
            self.packets.clear();
            self.sniffer = Some(sniffer);
        }
    }

    fn show_packets_button(&mut self, ui: &mut Ui) {
        let table_button_text = if self.is_packets_table_visible {
            "Hide packets"
        } else {
            "Show packets"
        };
        if ui.button(table_button_text).clicked() {
            self.is_packets_table_visible = !self.is_packets_table_visible
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

    fn show_or_hide_packets_window(&mut self, ctx: &Context) {
        if self.is_packets_table_visible {
            let mut packets_table = PacketsTable::new(
                &mut self.packets,
                &mut self.rtp_packet_ids,
                &mut self.rtcp_packet_ids,
            );
            packets_table.show(ctx, self.is_packets_table_visible);
        }
    }

    fn show_or_hide_rtp_packets_window(&mut self, ctx: &Context) {
        if self.is_rtp_packets_table_visible {
            let mut rtp_packets_table = RtpPacketsTable::new(&mut self.packets);
            rtp_packets_table.show(ctx, self.is_rtp_packets_table_visible);
        }
    }

    fn show_or_hide_streams_window(&mut self, ctx: &Context) {
        if self.is_streams_table_visible {
            let mut streams_table = StreamsTable::new(&self.packets, &mut self.is_jitter_visible);
            streams_table.show(ctx, self.is_streams_table_visible);
        }
    }
}
