use crate::streams::RefStreams;
use crate::utils::ntp_to_string;
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

        let mut last_id = 0;
        let mut next_ix = 1;

        let first_ts = streams.packets.get(0).unwrap().timestamp;
        body.heterogeneous_rows(heights, |ix, mut row| {
            let (id, rtcp) = rtcp_packets.get(ix).unwrap();
            let packet = streams.packets.get(*id).unwrap();

            if *id != last_id {
                last_id = *id;
                next_ix = 1;
            }

            row.col(|ui| {
                ui.label(format!("{} ({})", id, next_ix));
                next_ix += 1;
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
    // determined empirically
    let length = match packet {
        RtcpPacket::Goodbye(_) => 2.0,
        RtcpPacket::SourceDescription(sd) => {
            sd.chunks
                .iter()
                .map(|chunk| chunk.items.len() + 1)
                .max()
                // AFAIR, 0 chunk SDES packet is possible, although useless
                .unwrap_or(1) as f32
        }
        RtcpPacket::ReceiverReport(rr) => match rr.reports.len() {
            0 => 2.7,
            _ => 9.0,
        },
        RtcpPacket::SenderReport(sr) => match sr.reports.len() {
            0 => 4.7,
            _ => 11.0,
        },
        _ => 1.0,
    };

    length * 20.0
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
    build_label(ui, "Source:", format!("{:x}", report.ssrc));
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            let datetime = ntp_to_string(report.ntp_time);
            build_label(ui, "NTP time:", datetime);
            build_label(ui, "RTP time:", report.rtp_time.to_string());
        });
        ui.vertical(|ui| {
            build_label(ui, "Packet count:", report.packet_count.to_string());
            build_label(ui, "Octet count:", report.octet_count.to_string());
        });
    });
    ui.separator();
    build_reception_reports(ui, &report.reports);
}

fn build_receiver_report(ui: &mut Ui, report: &ReceiverReport) {
    build_label(ui, "Source:", format!("{:x}", report.ssrc));
    ui.separator();
    build_reception_reports(ui, &report.reports);
}

fn build_reception_reports(ui: &mut Ui, reports: &Vec<ReceptionReport>) {
    if reports.is_empty() {
        let text = RichText::new("No reception reports").strong();
        ui.label(text);
        return;
    }

    let mut first = true;
    ui.horizontal(|ui| {
        for report in reports {
            if !first {
                ui.separator();
            } else {
                first = false;
            }
            let fraction_lost = (report.fraction_lost as f64 / u8::MAX as f64) * 100.0;
            let delay = report.delay as f64 / u16::MAX as f64 * 1000.0;
            ui.vertical(|ui| {
                build_label(ui, "SSRC:", format!("{:x}", report.ssrc));
                build_label(ui, "Fraction lost:", format!("{}%", fraction_lost));
                build_label(ui, "Cumulative lost:", report.total_lost.to_string());
                build_label(
                    ui,
                    "Extended highest sequence number:",
                    report.last_sequence_number.to_string(),
                );
                build_label(ui, "Interarrival jitter:", report.jitter.to_string());
                build_label(
                    ui,
                    "Last SR timestamp:",
                    report.last_sender_report.to_string(),
                );
                build_label(ui, "Delay since last SR:", format!("{:.4} ms", delay));
            });
        }
    });
}

fn build_source_description(ui: &mut Ui, desc: &SourceDescription) {
    let mut first = true;
    ui.horizontal(|ui| {
        for chunk in &desc.chunks {
            if !first {
                ui.separator();
            } else {
                first = false;
            }
            ui.vertical(|ui| {
                build_label(ui, "Source:", format!("{:x}", chunk.source));
                for item in &chunk.items {
                    build_label(ui, item.sdes_type.to_string(), item.text.clone());
                }
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

    build_label(ui, "Sources:", ssrcs);
    build_label(ui, "Reason:", bye.reason.clone());
}

fn build_label(ui: &mut Ui, bold: impl Into<String>, normal: impl Into<String>) {
    let source_label = RichText::new(bold.into()).strong();
    ui.horizontal(|ui| {
        ui.label(source_label);
        ui.label(normal.into());
    });
}
