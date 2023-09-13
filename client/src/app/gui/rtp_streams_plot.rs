use std::collections::HashSet;
use std::fmt::{Display, Error, Formatter};

use eframe::egui;
use eframe::egui::plot::{Plot, Points};
use eframe::egui::{TextBuffer, Ui};
use eframe::epaint::Color32;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::packet::SessionProtocol::Rtp;
use rtpeeker_common::rtp::payload_type::MediaType;

use crate::app::gui::Packets;
use crate::app::gui::rtp_streams_plot::SettingsXAxis::RtpTimestamp;

#[derive(Debug)]
pub enum SettingsXAxis {
    RtpTimestamp,
    RawTimestamp,
    SequenceNumer,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RtpStreamsPlot {
    packets: Packets,
    settings_x_axis: SettingsXAxis,
    requires_reset: bool,
}


impl RtpStreamsPlot {

    pub fn new(
        packets: Packets,
    ) -> Self {
        Self {
            packets,
            settings_x_axis: RtpTimestamp,
            requires_reset: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let is_raw_timestamp = matches!(self.settings_x_axis, SettingsXAxis::RawTimestamp);
                let is_rtp_timestamp = matches!(self.settings_x_axis, SettingsXAxis::RtpTimestamp);
                let is_sequence_number = matches!(self.settings_x_axis, SettingsXAxis::SequenceNumer);

                if ui
                    .radio(is_raw_timestamp, "X axis is packet timestamp")
                    .clicked()
                {
                    self.settings_x_axis = SettingsXAxis::RawTimestamp;
                    self.requires_reset = true
                }
                if ui
                    .radio(is_rtp_timestamp, "X axis is RTP timestamp")
                    .clicked()
                {
                    self.settings_x_axis = SettingsXAxis::RtpTimestamp;
                    self.requires_reset = true
                }
                if ui
                    .radio(is_sequence_number, "X axis is sequence number")
                    .clicked()
                {
                    self.settings_x_axis = SettingsXAxis::SequenceNumer;
                    self.requires_reset = true
                }

                ui.separator();
                self.plot(ui);
            })
        });
    }

    fn plot(&mut self, ui: &mut Ui) {
        let mut set_of_ssrcs: HashSet<u32> = HashSet::new();
        let packets = self.packets.borrow();

        let rtp_packets: Vec<_> = packets
            .values()
            .filter(|packet| packet.session_protocol == Rtp)
            .collect();

        rtp_packets.iter().for_each(|packet| {
            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                unreachable!();
            };
            set_of_ssrcs.insert(rtp_packet.ssrc);
        });

        let mut ssrcs: Vec<_> = set_of_ssrcs.into_iter().collect();

        ssrcs.sort();


        let mut points: Vec<Points> = Vec::new();
        let mut points_xy: Vec<(f64, f64)> = Vec::new();

        ssrcs.iter().enumerate().for_each(| (stream_ix, ssrc)| {
            let stream_packets: Vec<_> = rtp_packets
                .iter()
                .filter(|packet| {
                    let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                        unreachable!();
                    };
                    rtp_packet.ssrc == *ssrc
                })
                .collect();

            stream_packets.iter().enumerate().for_each(|(packet_ix, packet)| {
                let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                    unreachable!();
                };

                let marker = rtp_packet.marker;
                let color = if marker { Color32::GREEN } else { Color32::RED };
                let additional_info = if marker {
                    match rtp_packet.payload_type.media_type {
                        MediaType::Audio => {
                            "For audio payload type, marker says that it is first packet after silence.\n"
                        }
                        MediaType::Video => {
                            "For video payload type, marker says that it is last packet of a video frame.\n"
                        }
                        MediaType::AudioOrVideo => "Marker could say that it is last packet of a video frame or \n\
                         that it is a first packet after silence.\n"
                    }
                } else {
                    "".as_str()
                };
                let mut on_hover = String::new();

                on_hover.push_str(&*format!(
                    "Source: {}\nDestination: {}\n",
                    packet.source_addr, packet.destination_addr
                ));
                // on_hover.push_str(&*rtp_packet.to_string());
                on_hover.push_str("\n");
                on_hover.push_str(&*rtp_packet.payload_type.to_string());
                on_hover.push_str("\n");
                on_hover.push_str(&*additional_info);

                let (x, y) = match self.settings_x_axis {
                    SettingsXAxis::RtpTimestamp => {
                        let y = if packet_ix == 0 {
                            stream_ix as f64
                        } else {
                            let last_packet = stream_packets.last().unwrap();
                            let SessionPacket::Rtp(ref last_rtp) = last_packet.contents else {
                                unreachable!();
                            };

                            let last_packet_timestamp = last_rtp.timestamp;
                            if rtp_packet.timestamp == last_packet_timestamp {
                                let y_shift = 0.01;
                                let last_packet_y = points_xy.last().unwrap().to_owned().1;
                                last_packet_y + y_shift
                            } else {
                                stream_ix as f64
                            }
                        };
                        (rtp_packet.timestamp as f64, y)
                    },
                    SettingsXAxis::RawTimestamp => { (packet.timestamp.as_secs_f64(), stream_ix as f64) },
                    SettingsXAxis::SequenceNumer => { (rtp_packet.sequence_number as f64, stream_ix as f64) },
                };

                on_hover.push_str(&*format!("x = {} [{}]\n", x, self.settings_x_axis));
                let point = Points::new([x, y]).name(on_hover).color(color).radius(1.5);

                points.push(point);
                points_xy.push((x, y ));
            });
        });

        let plot = Plot::new("halo")
            .show_background(false)
            .show_axes([true, false])
            .label_formatter(|name, _value| format!("{}", name))
            .view_aspect(2.0);

        if self.requires_reset {
            plot.reset().show(ui, |plot_ui| {
                for point in points {
                    plot_ui.points(point);
                }
            });
        } else {
            plot.show(ui, |plot_ui| {
                for point in points {
                    plot_ui.points(point);
                }
            });
        }
        self.requires_reset = false
    }
}

impl Display for SettingsXAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let name = match self {
            SettingsXAxis::RtpTimestamp => "RTP timestamp",
            SettingsXAxis::RawTimestamp => "Packet timestamp",
            SettingsXAxis::SequenceNumer => "Sequence number",
        };

        write!(f, "{}", name)
    }
}
