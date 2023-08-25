use super::Packets;
use egui_extras::{Column, TableBody, TableBuilder};

pub struct PacketsTable {
    packets: Packets,
}

impl PacketsTable {
    pub fn new(packets: Packets) -> Self {
        Self { packets }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn build_table(&self, ui: &mut egui::Ui) {
        let header_labels = vec![
            ("No.", "Packet number (including skipped packets)"),
            ("Time", "Packet arrival timestamp"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Protocol", "Transport layer protocol"),
            ("Length", "Length of the packet (including IP header)"),
            ("Treated as", "How was the UDP/TCP payload parsed"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::initial(40.0))
            .column(Column::initial(130.0))
            .columns(Column::initial(100.0), 2)
            .column(Column::initial(80.0))
            .column(Column::initial(80.0))
            .column(Column::initial(100.0))
            .header(20.0, |mut header| {
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

    fn build_table_body(&self, mut body: TableBody) {
        self.packets.borrow().iter().for_each(|(_id, packet)| {
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label(packet.id.to_string());
                });
                row.col(|ui| {
                    ui.label(packet.timestamp.as_secs_f64().to_string());
                });
                row.col(|ui| {
                    ui.label(packet.source_addr.to_string());
                });
                row.col(|ui| {
                    ui.label(packet.destination_addr.to_string());
                });
                row.col(|ui| {
                    ui.label(packet.transport_protocol.to_string());
                });
                row.col(|ui| {
                    ui.label(packet.length.to_string());
                });
                row.col(|ui| {
                    ui.label(packet.contents.to_string());
                });
            });
        })
    }
}
