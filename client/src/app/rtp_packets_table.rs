use super::is_stream_visible;
use crate::streams::{RefStreams, StreamKey};
use eframe::epaint::Color32;
use egui::RichText;
use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::packet::SessionPacket;
use std::collections::HashMap;

pub struct RtpPacketsTable {
    streams: RefStreams,
    streams_visibility: HashMap<StreamKey, bool>,
}

impl RtpPacketsTable {
    pub fn new(streams: RefStreams) -> Self {
        Self {
            streams,
            streams_visibility: HashMap::default(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.options_ui(ui);
            self.build_table(ui);
        });
    }

    fn options_ui(&mut self, ui: &mut egui::Ui) {
        let mut aliases = Vec::new();
        let streams = &self.streams.borrow().streams;
        let keys: Vec<_> = streams.keys().collect();

        keys.iter().for_each(|&key| {
            let alias = streams.get(key).unwrap().alias.to_string();
            aliases.push((key.clone(), alias));
        });
        aliases.sort_by(|(_, a), (_, b)| a.cmp(b));

        ui.horizontal_wrapped(|ui| {
            ui.label("Filter by: ");
            aliases.iter().for_each(|(key, alias)| {
                let selected = is_stream_visible(&mut self.streams_visibility, *key);
                ui.checkbox(selected, alias);
            });
        });
        ui.vertical(|ui| {
            ui.add_space(5.0);
        });
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
        let header_labels = [
            ("No.", "Packet number (including skipped packets)"),
            ("Time", "Packet arrival timestamp"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Padding", "RTP packet contains additional padding"),
            ("Extension", "RTP packet contains additional header extensions"),
            ("Marker", "RTP marker\nFor audio type it might say that it is first packet after silence\nFor video, marker might say that it is last packet of a frame"),
            ("Payload Type", "RTP payload type informs the receiver about the codec or encoding"),
            ("Sequence Number", "RTP sequence number ensures correct order and helps detect packet loss"),
            ("Timestamp", "RTP timestamp is the sender time of generating packet"),
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("Alias", "Locally assigned SSRC alias to make differentiating streams more convenient"),
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
            .columns(Column::remainder().at_least(80.0), 7)
            .column(Column::remainder().at_most(50.0))
            .columns(Column::remainder().at_least(80.0), 2)
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
        let streams = &self.streams.borrow();
        let rtp_packets: Vec<_> = streams
            .packets
            .values()
            .filter(|packet| {
                let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                    return false;
                };

                let key = (
                    packet.source_addr,
                    packet.destination_addr,
                    packet.transport_protocol,
                    rtp_packet.ssrc,
                );

                *is_stream_visible(&mut self.streams_visibility, key)
            })
            .collect();

        if rtp_packets.is_empty() {
            return;
        }

        let mut ssrc_to_display_name: HashMap<StreamKey, String> = HashMap::default();
        streams.streams.iter().for_each(|(key, stream)| {
            ssrc_to_display_name.insert(*key, stream.alias.to_string());
        });

        let first_ts = rtp_packets.get(0).unwrap().timestamp;
        body.rows(25.0, rtp_packets.len(), |row_ix, mut row| {
            let packet = rtp_packets.get(row_ix).unwrap();

            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                unreachable!();
            };

            let key = (
                packet.source_addr,
                packet.destination_addr,
                packet.transport_protocol,
                rtp_packet.ssrc,
            );

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
                ui.label(format_boolean(rtp_packet.padding));
            });
            row.col(|ui| {
                ui.label(format_boolean(rtp_packet.extension));
            });

            row.col(|ui| {
                ui.label(format_boolean(rtp_packet.marker));
            });

            let payload_type = &rtp_packet.payload_type;
            let (_, resp) = row.col(|ui| {
                ui.label(payload_type.id.to_string());
            });

            resp.on_hover_text(rtp_packet.payload_type.to_string());

            row.col(|ui| {
                // if rtp_packet.previous_packet_is_lost {
                //     let resp = ui.label(
                //         RichText::from(format!("{} ⚠", rtp_packet.sequence_number))
                //             .color(Color32::GOLD),
                //     );
                //     resp.on_hover_text(
                //         RichText::from("Previous packet is lost!").color(Color32::GOLD),
                //     );
                // } else {
                ui.label(rtp_packet.sequence_number.to_string());
                // }
            });

            row.col(|ui| {
                ui.label(rtp_packet.timestamp.to_string());
            });

            row.col(|ui| {
                ui.label(format!("{:x}", rtp_packet.ssrc));
            });
            row.col(|ui| {
                ui.label(ssrc_to_display_name.get(&key).unwrap().to_string());
            });

            row.col(|ui| {
                if rtp_packet.csrc.len() <= 1 {
                    let Some(csrc) = rtp_packet.csrc.first() else {
                        return;
                    };
                    ui.label(csrc.to_string());
                    return;
                }

                let formatted_csrc = rtp_packet
                    .csrc
                    .iter()
                    .map(|num| num.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");

                ui.label(format!("{:?}, ...", rtp_packet.csrc.first().unwrap()))
                    .on_hover_text(formatted_csrc);
            });

            row.col(|ui| {
                ui.label(rtp_packet.payload_length.to_string());
            });
        });
    }
}

fn format_boolean(value: bool) -> RichText {
    if value {
        RichText::from("✔").color(Color32::GREEN)
    } else {
        RichText::from("❌").color(Color32::RED)
    }
}
