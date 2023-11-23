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
            ("Expected number of packets", "Expected number of packets in stream, based on sequence numbers"),
            ("Packet loss", "Percentage of packets lost"),
            ("Duration", "Difference between last timestamp and first timestamp."),
            ("Mean jitter", "Average of jitter values for all of the packets"),
            ("Mean bitrate", "Average bitrate (sum of packets' lengts divided by stream duration)"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::initial(50.0).at_least(50.0))
            .column(Column::initial(100.0).at_least(100.0))
            .columns(Column::initial(150.0).at_least(150.0), 2)
            .columns(Column::initial(100.0).at_least(100.0), 2)
            .column(Column::initial(50.0).at_least(70.0))
            .columns(Column::initial(100.0).at_least(100.0), 3)
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
                ui.label(stream.get_expected_count().to_string());
            });
            row.col(|ui| {
                let lost = stream.get_expected_count() - stream.rtp_packets.len();
                let lost_fraction = lost as f64 / stream.get_expected_count() as f64;
                ui.label(format!("{:.3}%", lost_fraction * 100.0));
            });
            row.col(|ui| {
                let duration = stream.get_duration().as_secs_f64();
                ui.label(format!("{:.3}s", duration));
            });
            row.col(|ui| {
                let jitter = stream.get_mean_jitter() * 1000.0;
                ui.label(format!("{:.3}ms", jitter));
            });
            row.col(|ui| {
                ui.label(stream.get_mean_bitrate().to_string());
            });
        });
    }
}
