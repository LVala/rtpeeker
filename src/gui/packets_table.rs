use crate::sniffer::raw::{RawPacket, SessionPacket::*};
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
                    let packet = self.packets.get_mut(row_index).take().unwrap();
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

                    resp.context_menu(|ui| {
                        let s_pack = &packet.session_packet;
                        ui.label("Treat as:");
                        if ui.radio(matches!(s_pack, RTP(_)), "RTP").clicked() {
                            // TODO parse packets as RTP
                        } else if ui.radio(matches!(s_pack, RTCP(_)), "RTCP").clicked() {
                            // TODO parse packets as RTCP
                        } else if ui.radio(matches!(s_pack, Unknown), "Unknown").clicked() {
                            // TODO ignore these packets
                        }
                    });
                });
            });
    }
}
