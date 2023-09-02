use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::widgets::TextEdit;
use egui_extras::{Column, TableBody, TableBuilder};
use ewebsock::{WsMessage, WsSender};
use rtpeeker_common::{Request, RtpPacket};
use rtpeeker_common::packet::{Packet, SessionProtocol};
use rtpeeker_common::packet::SessionProtocol::Rtp;

use super::Packets;

type RtpPackets = Rc<RefCell<BTreeMap<usize, RtpPacket>>>;

pub struct RtpPacketsTable {
    rtp_packets: RtpPackets,
    filter_buffer: String,
}

impl RtpPacketsTable {
    pub fn new(packets: Packets) -> Self {
        let mut rtp_packets = RtpPackets::default();
        packets.borrow().iter().for_each(|(&ix, packet)| {
            if packet.session_protocol == Rtp {
                let Some(rtp_packet) = RtpPacket::build(packet); {
                    rtp_packets.insert(ix, rtp_packet);
                }
            }
        });

        Self {
            rtp_packets,
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
        let header_labels = [
            ("No.", "Packet number (including skipped packets)"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_least(40.0))
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
        let rtp_packets = self.rtp_packets.borrow();

        body.rows(25.0, rtp_packets.len(), |id, mut row| {
            // let first_ts = rtp_packets.get(&0).unwrap().timestamp;
            // let packet = rtp_packets.get(&id).unwrap();
            row.col(|ui| {
                ui.label(id.to_string());
            });
            // let timestamp = packet.timestamp - first_ts;
            // row.col(|ui| {
            //     ui.label(timestamp.as_secs_f64().to_string());
            // });
            // row.col(|ui| {
            //     ui.label(packet.source_addr.to_string());

            // resp.context_menu(|ui| {
            //     if let Some(req) = self.build_parse_menu(ui, packet) {
            //         requests.push(req);
            //     }
            // });
        });

        // cannot take mutable reference to self
        // unless `packets` is dropped, hence the `request` vector
        std::mem::drop(rtp_packets);
    }
}
