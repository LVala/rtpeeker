use egui::TextEdit;
use egui_extras::{Column, TableBody, TableBuilder};

use crate::streams::RefStreams;

pub struct RtpStreamsTable {
    streams: RefStreams,
}

impl RtpStreamsTable {
    pub fn new(streams: RefStreams) -> Self {
        Self { streams }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
        let header_labels = [
            ("Alias", "Locally assigned SSRC alias to make differentiating streams more convenient"),
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Number of packets", "Number of packets in stream"),
            ("Duration", "Difference between last timestamp and first timestamp."),
            ("Expected packets", "Difference between last timestamp and first timestamp."),
            ("Jitter", ""),
            ("Jitter plot", ""),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_most(70.0))
            .columns(Column::remainder().at_least(40.0), 7)
            .column(Column::remainder().at_least(380.0).resizable(false))
            .header(30.0, |mut header| {
                header_labels.iter().for_each(|(label, desc)| {
                    header.col(|ui| {
                        ui.heading(label.to_string())
                            .on_hover_text(desc.to_string());
                    });
                });
            })
            .body(|body| {
                self.build_table_body(body);
            });
    }

    fn build_table_body(&mut self, body: TableBody) {
        let mut streams = self.streams.borrow_mut();
        let keys: Vec<_> = streams.streams.keys().cloned().collect();

        body.rows(100.0, streams.streams.len(), |id, mut row| {
            let key = keys.get(id).unwrap();
            let stream = streams.streams.get_mut(key).unwrap();

            row.col(|ui| {
                let text_edit = TextEdit::singleline(&mut stream.alias).frame(false);
                ui.add(text_edit);
            });

            row.col(|ui| {
                ui.label(format!("{:x}", stream.ssrc));
            });
            row.col(|ui| {
                ui.label(stream.source_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream.destination_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream.rtp_packets.len().to_string());
            });
            row.col(|ui| {
                ui.label(format!("{:?}", stream.get_duration()));
            });
            row.col(|ui| {
                ui.label(format!("{:?}", stream.get_expected_count()));
            });
        });
    }
}
