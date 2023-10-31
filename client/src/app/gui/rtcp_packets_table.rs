use crate::streams::RefStreams;
use egui_extras::{Column, TableBody, TableBuilder};

pub struct RtcpPacketsTable {
    streams: RefStreams,
}

impl RtcpPacketsTable {
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
            ("No.", "Packet number (including skipped packets)"),
            ("Time", "Packet arrival timestamp"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Type", "Type of the RTCP packet"),
            ("Data", "Data specific to RTCP packet's type"),
        ];

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(80.0))
            .columns(Column::remainder().at_least(130.0), 2)
            .column(Column::remainder().at_least(80.0))
            .column(Column::remainder())
            .header(30.0, |mut header| {
                for (label, desc) in header_labels {
                    header.col(|ui| {
                        ui.heading(label.to_string())
                            .on_hover_text(desc.to_string());
                    });
                }
            })
            .body(|body| {
                self.build_table_body(body);
            });
    }

    fn build_table_body(&mut self, body: TableBody) {
        // TODO
    }
}
