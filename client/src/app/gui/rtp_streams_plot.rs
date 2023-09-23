use std::fmt::{Display, Error, Formatter};

use eframe::egui;
use eframe::egui::TextBuffer;
use eframe::epaint::Color32;
use egui::epaint::ahash::HashMap;
use egui::plot::{Plot, Points};
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::rtp::payload_type::MediaType;

use crate::app::gui::rtp_streams_plot::SettingsXAxis::{RawTimestamp, RtpTimestamp, SequenceNumer};
use crate::streams::RefStreams;

#[derive(Debug)]
pub enum SettingsXAxis {
    RtpTimestamp,
    RawTimestamp,
    SequenceNumer,
}

pub struct RtpStreamsPlot {
    streams: RefStreams,
    settings_x_axis: SettingsXAxis,
    requires_reset: bool,
    streams_visibility: HashMap<u32, bool>,
}

impl RtpStreamsPlot {
    pub fn new(streams: RefStreams) -> Self {
        Self {
            streams,
            settings_x_axis: RtpTimestamp,
            requires_reset: false,
            streams_visibility: HashMap::default(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        let streams = self.streams.borrow();
        let ssrcs: Vec<_> = streams.streams.keys().collect();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let is_raw_timestamp = matches!(self.settings_x_axis, RawTimestamp);
                        let is_rtp_timestamp = matches!(self.settings_x_axis, RtpTimestamp);
                        let is_sequence_number = matches!(self.settings_x_axis, SequenceNumer);

                        if ui
                            .radio(is_raw_timestamp, "X axis is packet timestamp")
                            .clicked()
                        {
                            self.settings_x_axis = RawTimestamp;
                            self.requires_reset = true
                        }
                        if ui
                            .radio(is_rtp_timestamp, "X axis is RTP timestamp")
                            .clicked()
                        {
                            self.settings_x_axis = RtpTimestamp;
                            self.requires_reset = true
                        }
                        if ui
                            .radio(is_sequence_number, "X axis is sequence number")
                            .clicked()
                        {
                            self.settings_x_axis = SequenceNumer;
                            self.requires_reset = true
                        }
                    });
                    ui.add_space(30.0);
                    ui.horizontal_wrapped(|ui| {
                        ssrcs.iter().for_each(|&ssrc| {
                            if !self.streams_visibility.contains_key(ssrc) {
                                self.streams_visibility.insert(*ssrc, true);
                            }
                            let selected = self.streams_visibility.get(ssrc).unwrap();

                            if ui
                                .radio(*selected, streams.streams.get(ssrc).unwrap().display_name.to_string())
                                .clicked()
                            {
                                let selected = self.streams_visibility.get(ssrc).unwrap();
                                self.streams_visibility.insert(*ssrc, !selected);
                            }
                        });
                    });
                });

                ui.separator();
                let packets = &streams.packets;
                let mut points: Vec<Points> = Vec::new();
                let mut points_xy: Vec<(f64, f64)> = Vec::new();

                streams.streams.iter().enumerate().for_each(|(stream_ix, (ssrc, stream))| {
                    if !(*self.streams_visibility.get(ssrc).unwrap()) {
                        return;
                    }

                    stream.rtp_packets.iter().enumerate().for_each(|(packet_ix, &ix)| {
                        let packet = packets.get(ix).unwrap();
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

                        on_hover.push_str(&stream.display_name);
                        on_hover.push('\n');
                        on_hover.push_str(&format!(
                            "Source: {}\nDestination: {}\n",
                            packet.source_addr, packet.destination_addr
                        ));
                        on_hover.push('\n');
                        on_hover.push_str(&rtp_packet.payload_type.to_string());
                        on_hover.push('\n');
                        on_hover.push_str(additional_info);

                        let (x, y) = match self.settings_x_axis {
                            RtpTimestamp => {
                                let y = if packet_ix == 0 {
                                    stream_ix as f64
                                } else {
                                    let last_packet = streams.packets.get(*stream.rtp_packets.last().unwrap()).unwrap();
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
                            }
                            RawTimestamp => { (packet.timestamp.as_secs_f64(), stream_ix as f64) }
                            SequenceNumer => { (rtp_packet.sequence_number as f64, stream_ix as f64) }
                        };

                        on_hover.push_str(&format!("x = {} [{}]\n", x, self.settings_x_axis));
                        let point = Points::new([x, y]).name(on_hover).color(color).radius(1.5);

                        points.push(point);
                        points_xy.push((x, y));
                    });
                });

                let plot = Plot::new("halo")
                    .show_background(false)
                    .show_axes([true, false])
                    .label_formatter(|name, _value| name.to_string())
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
            })
        });
    }
}

impl Display for SettingsXAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let name = match self {
            RtpTimestamp => "RTP timestamp",
            RawTimestamp => "Packet timestamp",
            SequenceNumer => "Sequence number",
        };

        write!(f, "{}", name)
    }
}
