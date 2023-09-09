use super::Packets;
use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::packet::{Packet, SessionProtocol};
use rtpeeker_common::Request;

pub struct RtpStreamsTable {
    packets: Packets,
}

impl RtpStreamsTable {
    pub fn new(packets: Packets) -> Self {
        Self { packets }
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
            ("Protocol", "Transport layer protocol"),
            ("Length", "Length of the packet (including IP header)"),
            ("Treated as", "How was the UDP/TCP payload parsed"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(130.0))
            .columns(Column::remainder().at_least(100.0), 2)
            .columns(Column::remainder().at_least(80.0), 2)
            .column(Column::remainder().at_least(100.0))
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
        let mut requests = Vec::new();
        let packets = self.packets.borrow();

        body.rows(25.0, packets.len(), |id, mut row| {
            let first_ts = packets.get(&0).unwrap().timestamp;
            let packet = packets.get(&id).unwrap();
            row.col(|ui| {
                ui.label(id.to_string());
            });
            let timestamp = packet.timestamp - first_ts;
            row.col(|ui| {
                ui.label(timestamp.as_secs_f64().to_string());
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
            let (_, resp) = row.col(|ui| {
                ui.label(packet.session_protocol.to_string());
            });

            resp.context_menu(|ui| {
                if let Some(req) = self.build_parse_menu(ui, packet) {
                    requests.push(req);
                }
            });
        });
    }

    fn build_parse_menu(&self, ui: &mut egui::Ui, packet: &Packet) -> Option<Request> {
        let mut request = None;
        ui.label(format!("Parse {} as:", &packet.id));
        SessionProtocol::all().iter().for_each(|packet_type| {
            let is_type = packet.session_protocol == *packet_type;
            if ui.radio(is_type, packet_type.to_string()).clicked() {
                request = Some(Request::Reparse(packet.id, *packet_type));
            }
        });
        ui.separator();
        ui.label("This will have effect on every client!");

        request
    }
}
