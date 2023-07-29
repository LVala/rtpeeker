use std::fmt::{Display, Error, Formatter};

use eframe::egui;
use eframe::egui::plot::{Plot, Points};
use eframe::egui::{Context, TextBuffer, Ui};
use eframe::epaint::Color32;
use egui::Window;

use crate::analysis::rtp::{Stream, Streams};
use crate::sniffer::raw::RawPacket;
use crate::sniffer::rtp::MediaType;

#[derive(Debug)]
pub enum SettingsXAxis {
    RtpTimestamp,
    RawTimestamp,
    SequenceNumer,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StreamsPlot<'a> {
    pub streams: Streams<'a>,
    settings_x_axis: &'a mut SettingsXAxis,
    requires_reset: bool,
}

impl<'a> StreamsPlot<'a> {
    pub fn new(
        rtp_packets: &'a [RawPacket],
        x_axis_is_rtp_timestamp: &'a mut SettingsXAxis,
    ) -> Self {
        let mut streams = Streams::new();

        for packet in rtp_packets {
            streams.add_packet(packet);
        }

        Self {
            streams,
            settings_x_axis: x_axis_is_rtp_timestamp,
            requires_reset: false,
        }
    }
}

impl StreamsPlot<'_> {
    fn header(&self) -> &'static str {
        "☰ RTP streams plot"
    }

    pub fn show(&mut self, ctx: &egui::Context, mut open: bool) {
        Window::new(self.header())
            .open(&mut open)
            .resizable(true)
            .default_width(1200.0)
            .show(ctx, |ui| {
                self.ui(ui, ctx);
            });
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.vertical(|ui| {
            let is_raw_timestamp = matches!(self.settings_x_axis, SettingsXAxis::RawTimestamp);
            let is_rtp_timestamp = matches!(self.settings_x_axis, SettingsXAxis::RtpTimestamp);
            let is_sequence_number = matches!(self.settings_x_axis, SettingsXAxis::SequenceNumer);

            if ui
                .radio(is_raw_timestamp, "X axis is packet timestamp")
                .clicked()
            {
                *self.settings_x_axis = SettingsXAxis::RawTimestamp;
                self.requires_reset = true
            }
            if ui
                .radio(is_rtp_timestamp, "X axis is RTP timestamp")
                .clicked()
            {
                *self.settings_x_axis = SettingsXAxis::RtpTimestamp;
                self.requires_reset = true
            }
            if ui
                .radio(is_sequence_number, "X axis is sequence number")
                .clicked()
            {
                *self.settings_x_axis = SettingsXAxis::SequenceNumer;
                self.requires_reset = true
            }

            ui.separator();
            self.plot(ui);
        });
    }

    fn plot(&mut self, ui: &mut Ui) {
        let mut points: Vec<Points> = Vec::new();
        let mut points_xy: Vec<(f64, f64)> = Vec::new();

        for stream_ix in 0..self.streams.streams.len() {
            let stream: &Stream = self.streams.streams.get(stream_ix).unwrap();

            for packet_ix in 0..stream.packets.len() {
                let packet = stream.packets.get(packet_ix).unwrap();
                let marker = packet.packet.header.marker;
                let color = if marker { Color32::GREEN } else { Color32::RED };
                let additional_info = if marker {
                    match packet.payload_type.media_type {
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
                    stream.source_addr, stream.destination_addr
                ));
                on_hover.push_str(&*packet.packet.to_string());
                on_hover.push_str("\n");
                on_hover.push_str(&*packet.payload_type.to_string());
                on_hover.push_str("\n");
                on_hover.push_str(&*additional_info);

                let (x, y) = match self.settings_x_axis {
                    SettingsXAxis::RtpTimestamp => {
                        let y = if packet_ix == 0 {
                            stream_ix as f64
                        } else {
                            let last_packet_ix = packet_ix - 1;
                            let last_packet_timestamp = stream.packets.get(last_packet_ix).unwrap().packet.header.timestamp;
                            if packet.packet.header.timestamp == last_packet_timestamp {
                                let y_shift = 0.01;
                                let last_packet_y = points_xy.last().unwrap().to_owned().1;
                                last_packet_y + y_shift
                            } else {
                                stream_ix as f64
                            }
                        };
                        (packet.packet.header.timestamp as f64, y)
                    },
                    SettingsXAxis::RawTimestamp => { (packet.raw_packet_timestamp.as_secs_f64(), stream_ix as f64) },
                    SettingsXAxis::SequenceNumer => { (packet.packet.header.sequence_number as f64, stream_ix as f64) },
                };

                on_hover.push_str(&*format!("x = {} [{}]\n", x, self.settings_x_axis));
                let point = Points::new([x, y]).name(on_hover).color(color).radius(1.5);

                points.push(point);
                points_xy.push((x, y ));
            }
        }

        let plot = Plot::new("halo").label_formatter(|name, _value| format!("{}", name)).view_aspect(2.0);

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
