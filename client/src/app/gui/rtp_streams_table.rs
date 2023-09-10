use std::collections::HashSet;
use std::ops::Div;
use std::time::Duration;

use eframe::egui::plot::{Line, Plot, PlotPoints};
use egui::{Color32, Vec2};
use egui::plot::{CoordinatesFormatter, Corner};
use egui_extras::{Column, TableBody, TableBuilder};
use rtpeeker_common::packet::{Packet, SessionPacket};
use rtpeeker_common::packet::SessionProtocol::Rtp;
use rtpeeker_common::RtpPacket;

use super::Packets;

pub struct RtpStreamsTable {
    packets: Packets,
}

impl RtpStreamsTable {
    pub fn new(packets: Packets) -> Self {
        Self {
            packets,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
        let header_labels = [
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Number of packets", "Number of packets in stream"),
            ("Duration", "Difference between last timestamp and first timestamp."),
            ("Lost packets", "Difference between last timestamp and first timestamp."),
            ("Jitter", ""),
            ("Jitter plot", ""),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(40.0))
            .column(Column::remainder().at_least(400.0).resizable(false))
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
        let mut set: HashSet<u32> = HashSet::new();

        let rtp_packets: Vec<_> = packets
            .values()
            .filter(|packet| packet.session_protocol == Rtp)
            .collect();

        rtp_packets.iter().for_each(|packet| {
            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                unreachable!();
            };
            set.insert(rtp_packet.ssrc);
        });

        let mut ssrcs: Vec<_> = set.into_iter().collect();
        ssrcs.sort();


        body.rows(100.0, ssrcs.len(), |id, mut row| {
            let stream_ssrc = ssrcs.get(id).unwrap();
            let stream_packets: Vec<_> = rtp_packets
                .iter()
                .filter(|packet| {
                    let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                        unreachable!();
                    };
                    rtp_packet.ssrc == *stream_ssrc
                })
                .collect();

            row.col(|ui| {
                ui.label(stream_ssrc.to_string());
            });
            row.col(|ui| {
                ui.label(stream_packets.first().unwrap().source_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream_packets.first().unwrap().destination_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream_packets.len().to_string());
            });
            row.col(|ui| {
                let duration = stream_packets.last().unwrap().timestamp.checked_sub(
                    stream_packets.first().unwrap().timestamp);

                if let Some(dur) = duration {
                    ui.label(format!("{:?}", dur));
                };
            });
            row.col(|ui| {
                let number_of_packets = stream_packets.len() as f64;
                if number_of_packets == 0.0 {
                    ui.label("0.00%");
                    return;
                }

                let first_packet = stream_packets.first().unwrap();
                let last_packet = stream_packets.last().unwrap();
                let SessionPacket::Rtp(ref first_rtp) = first_packet.contents else {
                    unreachable!();
                };
                let SessionPacket::Rtp(ref last_rtp) = last_packet.contents else {
                    unreachable!();
                };

                let first_sequence_number = first_rtp.sequence_number as f64;
                let last_sequence_number = last_rtp.sequence_number as f64;
                let expected_number_of_packets = last_sequence_number - first_sequence_number + 1.0;
                let lost_percentage = 100.0 - (number_of_packets / expected_number_of_packets) * 100.0;
                ui.label(format!("{:.2}%", lost_percentage));
            });
            let mut jitter_history = vec![0.0];
            row.col(|ui| {
                let mut jitter = 0.0;

                let mut last_packet: Option<&&&Packet> = None;
                stream_packets.iter().for_each(|packet| {
                    if let Some(prev_packet) = last_packet {
                        let SessionPacket::Rtp(ref rtp) = packet.contents else {
                            unreachable!();
                        };
                        let SessionPacket::Rtp(ref last_rtp) = prev_packet.contents else {
                            unreachable!();
                        };


                        if rtp.payload_type.clock_rate.is_none() || rtp.payload_type.id != last_rtp.payload_type.id {
                            jitter = 0.0;
                        } else {
                            let clock_rate = rtp.payload_type.clock_rate.unwrap();
                            let unit_timestamp = 1.0 / clock_rate as f64;
                            let arrival_time_difference_result = packet.timestamp.checked_sub(prev_packet.timestamp);
                            if let Some(arrival_time_difference) = arrival_time_difference_result {
                                let timestamp_difference = rtp.timestamp as f64 * unit_timestamp
                                    - last_rtp.timestamp as f64 * unit_timestamp;
                                let d = arrival_time_difference.as_secs_f64() - timestamp_difference;

                                jitter = jitter + (d - jitter) / 16.0;
                                jitter_history.push(jitter);
                            }
                        }
                        last_packet = Some(packet)
                    } else {
                        last_packet = Some(packet);
                    }
                });

                ui.label(jitter.to_string());
            });
            row.col(|ui| {
                ui.vertical_centered_justified(|ui| {
                    let points: PlotPoints = (0..jitter_history.len())
                        .map(|i| [i as f64, *jitter_history.get(i).unwrap()])
                        .collect();

                    let zero_axis: PlotPoints = (0..(jitter_history.len() + (jitter_history.len().div(5))))
                        .map(|i| {
                            [i as f64, 0.0]
                        })
                        .collect();

                    let line = Line::new(points).name("jitter");
                    let line_zero_axis = Line::new(zero_axis).color(Color32::GRAY);
                    Plot::new(id.to_string())
                        .show_background(false)
                        .show_axes([false, false])
                        .label_formatter(|name, value| {
                            if name.ne("jitter") || value.x.fract() != 0.0{
                                return format!("");
                            }
                            format!("no = {}\njitter = {:.5}", value.x, value.y)
                        })
                        .set_margin_fraction(Vec2::new(0.1, 0.1))
                        .allow_scroll(false)
                        .show(ui, |plot_ui| {
                            plot_ui.line(line_zero_axis);
                            plot_ui.line(line);
                        });
                    ui.add_space(7.0);
                });
            });
        });
    }

    // fn calculate_jitter(&mut self, last_jitter: f64, last_packet: &Packet, new_packet: &Packet) -> f64 {
    //     if let Some(clock_rate) = last_packet.payload_type.clock_rate_in_hz {
    //         if last_packet.packet.header.payload_type != new_packet.packet.header.payload_type {
    //             return 0.0;
    //         }
    //
    //         let unit_timestamp = 1.0 / clock_rate as f64;
    //
    //         let arrival_time_difference_result = new_packet.timestamp.checked_sub(last_packet.timestamp);
    //
    //         if let Some(arrival_time_difference) = arrival_time_difference_result {
    //             let timestamp_difference = new_packet.packet.header.timestamp as f64 * unit_timestamp
    //                 - last_packet.packet.header.timestamp as f64 * unit_timestamp;
    //             let d = arrival_time_difference.as_secs_f64() - timestamp_difference;
    //             return last_jitter + (d - last_jitter) / 16.0;
    //         }
    //         // self.jitter_history
    //         //     .push((self.jitter, arrival_timestamp.as_secs_f64()));
    //         0.0
    //     } else {
    //         0.0
    //     }
    // }
}
