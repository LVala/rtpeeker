use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::packet::SessionProtocol::Rtp;

use super::Packets;
use rtpeeker_common::rtp::get_payload_type_info;
pub struct RtpPacketsTable {
    packets: Packets,
}

impl RtpPacketsTable {
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
        let packets = self.packets.borrow();
        let rtp_packets: Vec<_> = packets
            .values()
            .filter(|packet| packet.session_protocol == Rtp)
            .collect();

        let first_ts = rtp_packets.get(0).unwrap().timestamp;
        body.rows(25.0, rtp_packets.len(), |row_ix, mut row| {
            let packet = rtp_packets.get(row_ix).unwrap();

            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                unreachable!();
            };

            row.col(|ui| {
                ui.label(packet.id.to_string());
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

            let (_, resp) = row.col(|ui| {
                ui.label(rtp_packet.payload_type.to_string());
            });
            let (name, media_type, clock_rate_in_hz) = get_payload_type_info(rtp_packet.payload_type);

            resp.on_hover_text(format!(
                "Payload type: {}\n\
                Name: {}\n\
                Type: {}\n\
                Clock rate: {} Hz\n",
                rtp_packet.payload_type, name, media_type, clock_rate_in_hz
            ));

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
                    let formatted_csrc = rtp_packet
                        .csrc
                        .iter()
                        .map(|num| num.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");

                    ui.label(format!("{:?}, ...", rtp_packet.csrc.first().unwrap()))
                        .on_hover_text(formatted_csrc);
                }
            });

            row.col(|ui| {
                ui.label(rtp_packet.payload_length.to_string());
            });
        });
    }
}
