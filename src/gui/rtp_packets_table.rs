use crate::sniffer::raw::{RawPacket, SessionPacket::RTP};
use crate::sniffer::rtp::RtpPacket;
use eframe::egui;
use eframe::egui::Ui;
use egui::Window;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RtpPacketsTable<'a> {
    scroll_to_row_slider: usize,
    scroll_to_row: Option<usize>,
    packets: &'a mut Vec<RawPacket>,
}

impl<'a> RtpPacketsTable<'a> {
    pub fn new(packets: &'a mut Vec<RawPacket>) -> Self {
        Self {
            packets,
            scroll_to_row: None,
            scroll_to_row_slider: 0,
        }
    }
}

impl RtpPacketsTable<'_> {
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
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.sort_by_sequence_number_button(ui);
                self.sort_by_time_stamp_button(ui);
            });

            self.slider(ui);
        });

        ui.separator();

        self.table(ui);
    }

    fn sort_by_sequence_number_button(&mut self, ui: &mut Ui) {
        if ui.button("Sort by sequence number").clicked() {
            // self.packets.sort_by(|a, b| {
            //     a.packet
            //         .header
            //         .sequence_number
            //         .partial_cmp(&b.packet.header.sequence_number)
            //         .unwrap()
            // })
        }
    }

    fn sort_by_time_stamp_button(&mut self, ui: &mut Ui) {
        if ui.button("Sort by time stamp").clicked() {
            // self.rtp_packets.sort_by(|a, b| {
            //     a.packet
            //         .header
            //         .timestamp
            //         .partial_cmp(&b.packet.header.timestamp)
            //         .unwrap()
            // })
        }
    }

    fn slider(&mut self, ui: &mut Ui) {
        let length = self
            .packets
            .iter()
            .filter(|packet| matches!(packet.session_packet, RTP(_)))
            .count();

        let slider_response = ui.add(
            egui::Slider::new(&mut self.scroll_to_row_slider, 0..=length)
                .logarithmic(true)
                .text("Row to scroll to"),
        );
        if slider_response.changed() {
            self.packets.len();
            self.scroll_to_row = Some(self.scroll_to_row_slider);
        }
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
        let packets: Vec<_> = self
            .packets
            .iter()
            .filter(|packet| matches!(packet.session_packet, RTP(_)))
            .collect();

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
            .column(Column::initial(70.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(70.0).at_least(40.0).clip(true))
            .column(Column::initial(70.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
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
                    ui.strong("Destination Address");
                });
                header.col(|ui| {
                    ui.strong("Source Address");
                });
                header.col(|ui| {
                    ui.strong("Version");
                });
                header.col(|ui| {
                    ui.strong("Padding");
                });
                header.col(|ui| {
                    ui.strong("Extension");
                });
                header.col(|ui| {
                    ui.strong("Marker");
                });
                header.col(|ui| {
                    ui.strong("Payload type");
                });
                header.col(|ui| {
                    ui.strong("Sequence number");
                });
                header.col(|ui| {
                    ui.strong("Time stamp");
                });
                header.col(|ui| {
                    ui.strong("SSRC");
                });
                header.col(|ui| {
                    ui.strong("CSRC");
                });
                header.col(|ui| {
                    ui.strong("Extension profile");
                });
                header.col(|ui| {
                    ui.strong("Extensions");
                });
                header.col(|ui| {
                    ui.strong("Payload");
                });
            })
            .body(|body| {
                body.rows(text_height, packets.len(), |row_index, mut row| {
                    let packet = packets.get(row_index).take().unwrap();
                    let RTP(RtpPacket{packet: ref rtp_packet, ..}) = packet.session_packet else {
                        return;
                    };

                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.destination_addr.to_string());
                    });
                    row.col(|ui| {
                        ui.label(packet.source_addr.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.version.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.padding.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.extension.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.marker.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.payload_type.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.sequence_number.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.timestamp.to_string());
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.ssrc.to_string());
                    });
                    row.col(|ui| {
                        ui.label(format!("{:?}", rtp_packet.header.csrc));
                    });
                    row.col(|ui| {
                        ui.label(rtp_packet.header.extension_profile.to_string());
                    });
                    row.col(|ui| {
                        ui.label(format!("{:?}", rtp_packet.header.extensions));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:?}", rtp_packet.payload));
                    });
                });
            });
    }
}
