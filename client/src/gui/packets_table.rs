use super::Packets;
use egui::widgets::TextEdit;
use egui_extras::{Column, TableBody, TableBuilder};
use ewebsock::{WsMessage, WsSender};
use rtpeeker_common::packet::{Packet, PacketType};
use rtpeeker_common::Request;

pub struct PacketsTable {
    packets: Packets,
    ws_sender: WsSender,
    filter_buffer: String,
}

impl PacketsTable {
    pub fn new(packets: Packets, ws_sender: WsSender) -> Self {
        Self {
            packets,
            ws_sender,
            filter_buffer: String::new(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("filter_bar").show(ctx, |ui| {
            self.build_filter(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn build_filter(&mut self, ui: &mut egui::Ui) {
        let text_edit = TextEdit::singleline(&mut self.filter_buffer)
            .font(egui::style::TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .hint_text("Apply a filter ...");

        ui.horizontal(|ui| {
            // TODO: implement the actuall filtering
            ui.button("↻").on_hover_text("Reset the filter");
            ui.button("⏵").on_hover_text("Apply the filter");
            ui.add(text_edit);
        });
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
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
            let packet = packets.get(&id).unwrap();
            row.col(|ui| {
                ui.label(id.to_string());
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
            let (_, resp) = row.col(|ui| {
                ui.label(packet.session_protocol.to_string());
            });

            resp.context_menu(|ui| {
                if let Some(req) = self.build_parse_menu(ui, packet) {
                    requests.push(req);
                }
            });
        });

        // cannot take mutable reference to self
        // unless `packets` is dropped, hence the `request` vector
        std::mem::drop(packets);
        requests
            .iter()
            .for_each(|req| self.send_parse_request(*req));
    }

    fn build_parse_menu(&self, ui: &mut egui::Ui, packet: &Packet) -> Option<Request> {
        let mut request = None;
        ui.label(format!("Parse {} as:", &packet.id));
        PacketType::all().iter().for_each(|packet_type| {
            let is_type = packet.session_protocol == *packet_type;
            if ui.radio(is_type, packet_type.to_string()).clicked() {
                request = Some(Request::Reparse(packet.id, *packet_type));
            }
        });
        ui.separator();
        ui.label("This will have effect on every client!");

        request
    }

    fn send_parse_request(&mut self, request: Request) {
        let Ok(msg) = request.encode() else {
            log::error!("Failed to encode a request message");
            return;
        };
        let msg = WsMessage::Binary(msg);

        self.ws_sender.send(msg);
    }
}
