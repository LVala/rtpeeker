use crate::sniffer::raw::{RawPacket, TransportProtocol, SessionPacket::*};
use crate::sniffer::rtp::RtpPacket;
use crate::sniffer::rtcp::RtcpPacketGroup;
use std::net::SocketAddr;
use eframe::egui;
use eframe::egui::Ui;
use egui::Window;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PacketsTable<'a> {
    scroll_to_row: Option<usize>,
    packets: &'a mut Vec<RawPacket>,
}

impl<'a> PacketsTable<'a> {
    pub fn new(packets: &'a mut Vec<RawPacket>) -> Self {
        Self {
            packets,
            scroll_to_row: None,
        }
    }
}

impl PacketsTable<'_> {
    fn header(&self) -> &'static str {
        "☰ RTP packets"
    }

    pub fn show(&mut self, ctx: &egui::Context, mut open: bool) {
        Window::new(self.header())
            .open(&mut open)
            .resizable(true)
            .default_width(1200.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }

    fn ui(&mut self, ui: &mut Ui) {
        self.table(ui);
    }

    fn table(&mut self, ui: &mut Ui) {
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0))
            .size(Size::exact(10.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        self.table_ui(ui);
                    });
                });
            });
    }

    fn table_ui(&mut self, ui: &mut Ui) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(100.0).range(40.0..=300.0))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(70.0).at_least(40.0).clip(true))
            .column(Column::initial(70.0).at_least(40.0).clip(true))
            .column(Column::initial(70.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(0.0);

        if let Some(row_nr) = self.scroll_to_row.take() {
            table = table.scroll_to_row(row_nr, None);
        }

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Row");
                });
                header.col(|ui| {
                    ui.strong("Timestamp");
                });
                header.col(|ui| {
                    ui.strong("Destination Address");
                });
                header.col(|ui| {
                    ui.strong("Source Address");
                });
                header.col(|ui| {
                    ui.strong("Length");
                });
                header.col(|ui| {
                    ui.strong("Transport Protocol");
                });
                header.col(|ui| {
                    ui.strong("Session Protocol");
                });
            })
            .body(|body| {
                body.rows(text_height, self.packets.len(), |row_index, mut row| {
                    let packet = self.packets.get(row_index).take().unwrap();
                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.timestamp.as_secs_f64().to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.destination_addr.to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.source_addr.to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.length.to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.transport_protocol.to_string());
                    });
                    let (_, resp) = row.col(|ui| {
                        ui.label(packet.session_packet.to_string());
                    });

                    let source = packet.source_addr;
                    let dest = packet.destination_addr;
                    let protocol = packet.transport_protocol;
                    let is_rtp = matches!(packet.session_packet, RTP(_));
                    let is_rtcp = matches!(packet.session_packet, RTCP(_));

                    resp.context_menu(|ui| {
                        ui.label("Treat as:");
                        if ui.radio(is_rtp, "RTP").clicked() {
                            parse_packets_as(self.packets, source, dest, protocol, true, false);
                        }
                        if ui.radio(is_rtcp, "RTCP").clicked() {
                            parse_packets_as(self.packets, source, dest, protocol, false, true);
                        }
                        if ui.radio(!is_rtp && !is_rtcp, "Unknown").clicked() {
                            parse_packets_as(self.packets, source, dest, protocol, false, false);
                        }
                    });
                });
            });
    }
}

fn parse_packets_as(packets: &mut Vec<RawPacket>, source: SocketAddr, dest: SocketAddr, protocol: TransportProtocol, is_rtp: bool, is_rtcp: bool) {
    for pack in packets.iter_mut() {
        if source == pack.source_addr 
          && dest == pack.destination_addr 
          && protocol == pack.transport_protocol {
            if is_rtp {
                let rtp_packet = RtpPacket::build(pack).unwrap();
                pack.session_packet = RTP(rtp_packet);
            } else if is_rtcp {
                let rtcp_packets = RtcpPacketGroup::rtcp_packets_from(pack).unwrap();
                pack.session_packet = RTCP(rtcp_packets);
            } else {
                pack.session_packet = Unknown;
            }
        }
    }
}
