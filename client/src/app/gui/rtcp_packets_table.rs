use crate::streams::RefStreams;
use egui::{RichText, Ui};
use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::rtcp::*;
use rtpeeker_common::{packet::SessionPacket, RtcpPacket};

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
            ("No.", "Packet number (including skipped packets) + compound RTCP packet number inside the parentheses"),
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
            .column(Column::initial(60.0).at_least(60.0))
            .column(Column::initial(70.0).at_least(70.0))
            .columns(Column::initial(150.0).at_least(150.0), 2)
            .column(Column::initial(170.0).at_least(170.0))
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
        let streams = &self.streams.borrow();
        let rtcp_packets: Vec<_> = streams
            .packets
            .values()
            .filter_map(|packet| {
                let rtcp = match packet.contents {
                    SessionPacket::Rtcp(ref rtcp) => rtcp,
                    _ => return None,
                };

                Some(rtcp.iter().map(|pack| (packet.id, pack)))
            })
            .flatten()
            .collect();

        if rtcp_packets.is_empty() {
            return;
        }

        let heights = rtcp_packets.iter().map(|(_, pack)| get_row_height(pack));

        let (id, _) = rtcp_packets.get(0).unwrap();
        let mut last_id = *id;
        let mut next_ix = 1;

        let first_ts = streams.packets.get(0).unwrap().timestamp;
        body.heterogeneous_rows(heights, |ix, mut row| {
            let (id, rtcp) = rtcp_packets.get(ix).unwrap();
            let packet = streams.packets.get(*id).unwrap();

            let sub_ix = if *id == last_id {
                next_ix += 1;
                next_ix - 1
            } else {
                last_id = *id;
                next_ix = 1;
                next_ix
            };

            row.col(|ui| {
                ui.label(format!("{} ({})", id, sub_ix));
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
                ui.label(rtcp.get_type_name().to_string());
            });
            row.col(|ui| {
                build_packet(ui, rtcp);
            });
        });
    }
}

fn get_row_height(packet: &RtcpPacket) -> f32 {
    let length = match packet {
        RtcpPacket::Goodbye(_) => 2,
        RtcpPacket::SourceDescription(sd) => {
            sd.chunks.iter().map(|chunk| chunk.items.len() + 1).sum()
        }
        RtcpPacket::ReceiverReport(rr) => 7 * rr.reports.len() + 1,
        RtcpPacket::SenderReport(sr) => 7 * sr.reports.len() + 6,
        _ => 1,
    };

    length as f32 * 17.0
}

fn build_packet(ui: &mut Ui, packet: &RtcpPacket) {
    match packet {
        RtcpPacket::SenderReport(report) => build_sender_report(ui, report),
        RtcpPacket::ReceiverReport(report) => build_receiver_report(ui, report),
        RtcpPacket::SourceDescription(desc) => build_source_description(ui, desc),
        RtcpPacket::Goodbye(bye) => build_goodbye(ui, bye),
        _ => {
            ui.label("Unsupported");
        }
    };
}

fn build_sender_report(ui: &mut Ui, report: &SenderReport) {
    ui.label(format!("Source: {}", report.ssrc));
    ui.label(format!("NTP time: {}", report.ntp_time));
    ui.label(format!("RTP time: {}", report.rtp_time));
    ui.label(format!("Packet count: {}", report.packet_count));
    ui.label(format!("Octet count: {}", report.octet_count));
    build_reception_reports(ui, &report.reports);
}

fn build_receiver_report(ui: &mut Ui, report: &ReceiverReport) {
    ui.label(format!("Source: {}", report.ssrc));
    build_reception_reports(ui, &report.reports);
}

fn build_reception_reports(ui: &mut Ui, reports: &Vec<ReceptionReport>) {
    ui.label("Reports:");
    for report in reports {
        ui.label(format!("SSRC: {}", report.ssrc));
        ui.label(format!("Fraction lost: {}", report.fraction_lost));
        ui.label(format!("Cumulative lost: {}", report.total_lost));
        ui.label(format!("Interarrival jitter: {}", report.jitter));
        ui.label(format!("Last sr: {}", report.last_sender_report));
        ui.label(format!("Delay last sr: {}", report.delay));
    }
}

fn build_source_description(ui: &mut Ui, desc: &SourceDescription) {
    desc.chunks.iter().for_each(|chunk| {
        let source_label = RichText::new("Source: ").strong();
        ui.horizontal(|ui| {
            ui.label(source_label);
            ui.label(format!("{:x}", chunk.source));
        });
        for item in &chunk.items {
            let type_label = RichText::new(item.sdes_type.to_string()).strong();
            ui.horizontal(|ui| {
                ui.label("   •");
                ui.label(type_label);
                ui.label(format!(" {}", item.text));
            });
        }
    });
}

fn build_goodbye(ui: &mut Ui, bye: &Goodbye) {
    let ssrcs = bye
        .sources
        .iter()
        .map(|ssrc| format!("{:x}", ssrc))
        .collect::<Vec<_>>()
        .join(", ");

    let label_sources = RichText::new("Sources: ").strong();
    ui.horizontal(|ui| {
        ui.label(label_sources);
        ui.label(ssrcs);
    });

    let label_reason = RichText::new("Reason: ").strong();
    ui.horizontal(|ui| {
        ui.label(label_reason);
        ui.label(bye.reason.to_string());
    });
}
