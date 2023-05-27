use crate::analysis::rtp::Streams;
use crate::sniffer::rtp::RtpPacket;
use eframe::egui;
use eframe::egui::Ui;
use egui::Window;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StreamsTable<'a> {
    pub streams: Streams<'a>,
}

impl<'a> StreamsTable<'a> {
    pub fn new(rtp_packets: &'a [RtpPacket]) -> Self {
        let mut streams = Streams::new();

        for packet in rtp_packets {
            streams.add_packet(packet);
        }

        Self { streams }
    }
}

impl StreamsTable<'_> {
    fn header(&self) -> &'static str {
        "☰ RTP streams"
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

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(100.0).range(40.0..=300.0))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(0.0);

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
                    ui.strong("SSRC");
                });
                header.col(|ui| {
                    ui.strong("Duration (s)");
                });
                header.col(|ui| {
                    ui.strong("Payload type");
                });
                header.col(|ui| {
                    ui.strong("Number of packets");
                });
                header.col(|ui| {
                    ui.strong("Packet loss");
                });
                header.col(|ui| {
                    ui.strong("Jitter");
                });
            })
            .body(|body| {
                body.rows(
                    text_height,
                    self.streams.streams.len(),
                    |row_index, mut row| {
                        let rtp_stream = self.streams.streams.get(row_index).take().unwrap();
                        row.col(|ui| {
                            ui.label(row_index.to_string());
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.destination_addr.to_string());
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.source_addr.to_string());
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.ssrc.to_string());
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.duration().as_secs_f64().to_string());
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.payload_type.to_string());
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.num_of_packets().to_string());
                        });
                        row.col(|ui| {
                            let packet_loss_perc = rtp_stream.lost_packets_percentage();
                            let packet_loss = format!("{}%", packet_loss_perc);
                            ui.label(packet_loss);
                        });
                        row.col(|ui| {
                            ui.label(rtp_stream.jitter().to_string());
                        });
                    },
                );
            });
    }
}
