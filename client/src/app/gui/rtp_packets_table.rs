use egui::widgets::TextEdit;
use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::packet::SessionProtocol::Rtp;

use super::payload_type::PayloadType;

use super::Packets;

pub struct RtpPacketsTable {
    packets: Packets,
    filter_buffer: String,
}

impl RtpPacketsTable {
    pub fn new(packets: Packets) -> Self {
        Self {
            packets,
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
            ("Time", "Packet arrival timestamp"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Version", "RTP protocol version"),
            ("Padding", "RTP packet contains additional padding"),
            ("Extension", "RTP packet contains additional header extensions"),
            ("Marker", "RTP marker\nFor audio type it might say that it is first packet after silence\nFor video, marker might say that it is last packet of a frame"),
            ("Payload Type", "RTP payload type informs the receiver about the codec or encoding"),
            ("Sequence Number", "RTP sequence number ensures correct order and helps detect packet loss"),
            ("Timestamp", "RTP timestamp is the sender time of generating packet"),
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("CSRC", "RTP CSRC (Contributing Source Identifier)\nSSRC identifiers of the sources that have contributed to a composite RTP packet,\ntypically used for audio mixing in conferences."),
            ("Payload Length", "RTP payload length (Excluding header and extensions)"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(80.0))
            .columns(Column::remainder().at_least(130.0), 2)
            .columns(Column::remainder().at_least(80.0), 10)
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
        let mut rtp_packets_ids = Vec::new();
        self.packets.borrow().iter().for_each(|(&ix, packet)| {
            if packet.session_protocol == Rtp {
                rtp_packets_ids.push(ix);
            }
        });
        let packets = self.packets.borrow();

        body.rows(25.0, rtp_packets_ids.len(), |row_ix, mut row| {
            let first_rtp_id = rtp_packets_ids.first().unwrap();
            let first_ts = packets.get(first_rtp_id).unwrap().timestamp;
            let rtp_id = rtp_packets_ids.get(row_ix).unwrap();

            let packet = packets.get(rtp_id).unwrap();
            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                panic!("Error. This should be RTP");
            };

            row.col(|ui| {
                ui.label(row_ix.to_string());
            });
            row.col(|ui| {
                let timestamp = packet.timestamp - first_ts;
                ui.label(format!("{:.4}", timestamp.as_secs_f64()));
            });
            row.col(|ui| {
                ui.label(packet.source_addr.to_string());
            });
            row.col(|ui| {
                ui.label(packet.destination_addr.to_string());
            });
            row.col(|ui| {
                ui.label(rtp_packet.version.to_string());
            });

            row.col(|ui| {
                ui.label(rtp_packet.padding.to_string());
            });
            row.col(|ui| {
                ui.label(rtp_packet.extension.to_string());
            });

            row.col(|ui| {
                ui.label(rtp_packet.marker.to_string());
            });

            let resp = row.col(|ui| {
                ui.label(rtp_packet.payload_type.to_string());
            });
            resp.1
                .on_hover_text(PayloadType::new(rtp_packet.payload_type).to_string());

            row.col(|ui| {
                ui.label(rtp_packet.sequence_number.to_string());
            });

            row.col(|ui| {
                ui.label(rtp_packet.timestamp.to_string());
            });

            row.col(|ui| {
                ui.label(rtp_packet.ssrc.to_string());
            });

            row.col(|ui| {
                if !rtp_packet.csrc.is_empty() {
                    ui.label(format!("{:?}, ...", rtp_packet.csrc.first().unwrap()))
                        .on_hover_text(format!("{:?}", rtp_packet.csrc));
                }
            });

            row.col(|ui| {
                ui.label(rtp_packet.payload_length.to_string());
            });
        });
    }
}
