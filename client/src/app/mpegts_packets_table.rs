use super::is_stream_visible;
use crate::streams::RefStreams;
use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::mpegts::header::PIDTable;
use rtpeeker_common::mpegts::FRAGMENT_SIZE;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::StreamKey;
use std::collections::HashMap;

pub struct MpegTsPacketsTable {
    streams: RefStreams,
    streams_visibility: HashMap<StreamKey, bool>,
}

impl MpegTsPacketsTable {
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
        let streams = &self.streams.borrow().mpeg_ts_streams;
        let keys: Vec<_> = streams.keys().collect();

        keys.iter().for_each(|&key| {
            let alias = streams.get(key).unwrap().alias.to_string();
            aliases.push((*key, alias));
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
            ("P1", "Packet No. 1"),
            ("P2", "Packet No. 2"),
            ("P3", "Packet No. 3"),
            ("P4", "Packet No. 4"),
            ("P5", "Packet No. 5"),
            ("P6", "Packet No. 6"),
            ("P7", "Packet No. 7"),
            ("Payload Length", "Mpegts payload length"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(80.0))
            .columns(Column::remainder().at_least(130.0), 2)
            .columns(Column::remainder().at_least(120.0), 7)
            .column(Column::remainder().at_least(80.0))
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
        let mpegts_packets: Vec<_> = streams
            .packets
            .values()
            .filter(|packet| {
                let SessionPacket::Mpegts(ref mpegts_packet) = packet.contents else {
                    return false;
                };

                let key = (
                    packet.source_addr,
                    packet.destination_addr,
                    packet.transport_protocol,
                    0, //TODO: change
                );

                *is_stream_visible(&mut self.streams_visibility, key)
            })
            .collect();

        if mpegts_packets.is_empty() {
            return;
        }

        let mut ssrc_to_display_name: HashMap<StreamKey, String> = HashMap::default();
        streams.mpeg_ts_streams.iter().for_each(|(key, stream)| {
            ssrc_to_display_name.insert(*key, stream.alias.to_string());
        });

        let first_ts = mpegts_packets.first().unwrap().timestamp;

        body.rows(25.0, mpegts_packets.len(), |row_ix, mut row| {
            let packet = mpegts_packets.get(row_ix).unwrap();

            let SessionPacket::Mpegts(ref mpegts_packet) = packet.contents else {
                unreachable!();
            };

            let key = (
                packet.source_addr,
                packet.destination_addr,
                packet.transport_protocol,
                0, //TODO: change
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

            let mut labels = mpegts_packet
                .fragments
                .iter()
                .map(|fragment| match fragment.header.pid {
                    PIDTable::ProgramAssociation => String::from("ProgramAssociation"),
                    PIDTable::ConditionalAccess => String::from("ConditionalAccess"),
                    PIDTable::TransportStreamDescription => {
                        String::from("TransportStreamDescription")
                    }
                    PIDTable::AdaptiveStreamingInformation => {
                        String::from("AdaptiveStreamingInformation")
                    }
                    PIDTable::NullPacket => String::from("NullPacket"),
                    PIDTable::PID(pid) => format!("PID: {}", pid),
                    PIDTable::IPMPControlInformation => String::from("IPMPControlInformation"),
                })
                .into_iter();

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label(labels.next().unwrap_or_default());
            });

            row.col(|ui| {
                ui.label((mpegts_packet.number_of_fragments * FRAGMENT_SIZE).to_string());
            });
        });
    }
}
