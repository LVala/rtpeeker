use std::cell::Ref;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};

use eframe::egui;
use eframe::egui::TextBuffer;
use eframe::epaint::Color32;
use egui::plot::{Line, Plot, PlotPoints, PlotUi, Points};
use egui::Ui;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::rtp::payload_type::MediaType;
use rtpeeker_common::{Packet, RtpPacket};

use crate::streams::{is_stream_visible, RefStreams, Streams};

use self::SettingsXAxis::*;

struct PointData {
    x: f64,
    y: f64,
    height: f64,
    on_hover: String,
    color: Color32,
    radius: f32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum SettingsXAxis {
    RtpTimestamp,
    RawTimestamp,
    SequenceNumer,
}

impl SettingsXAxis {
    fn all() -> Vec<Self> {
        vec![RtpTimestamp, RawTimestamp, SequenceNumer]
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

pub struct RtpStreamsPlot {
    streams: RefStreams,
    points_data: Vec<PointData>,
    settings_x_axis: SettingsXAxis,
    requires_reset: bool,
    streams_visibility: HashMap<u32, bool>,
    last_rtp_packets_len: usize,
}

impl RtpStreamsPlot {
    pub fn new(streams: RefStreams) -> Self {
        Self {
            streams,
            points_data: Vec::new(),
            settings_x_axis: RtpTimestamp,
            requires_reset: false,
            streams_visibility: HashMap::default(),
            last_rtp_packets_len: 0,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.collapsing("Settings", |ui| {
                self.options_ui(ui);
            });
            self.plot_ui(ui);
        });
    }

    fn options_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("X axis value:");
            SettingsXAxis::all().into_iter().for_each(|setting| {
                if ui
                    .radio(setting == self.settings_x_axis, setting.to_string())
                    .clicked()
                {
                    self.settings_x_axis = setting;
                    self.requires_reset = true;
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            let streams = &self.streams.borrow().streams;
            let ssrcs: Vec<_> = streams.keys().collect();

            ui.label("Toggle streams: ");
            ssrcs.iter().for_each(|&ssrc| {
                let selected = is_stream_visible(&mut self.streams_visibility, *ssrc);
                let resp = ui.checkbox(
                    selected,
                    streams.get(ssrc).unwrap().display_name.to_string(),
                );
                if resp.clicked() {
                    self.requires_reset = true
                }
            });
        });
        ui.vertical(|ui| {
            ui.add_space(10.0);
        });
    }

    fn plot_ui(&mut self, ui: &mut Ui) {
        let number_of_rtp_packets = self.number_of_rtp_packets();
        if self.last_rtp_packets_len != number_of_rtp_packets || self.requires_reset {
            self.refresh_points();
        }

        let plot = Plot::new("rtp-plot")
            .show_background(false)
            .show_axes([true, false])
            .label_formatter(|name, _value| name.to_string());

        if self.requires_reset {
            plot.reset().show(ui, |plot_ui| {
                self.draw_points(plot_ui);
            });
        } else {
            plot.show(ui, |plot_ui| {
                self.draw_points(plot_ui);
            });
        }
        self.requires_reset = false;
        self.last_rtp_packets_len = number_of_rtp_packets;
    }

    fn number_of_rtp_packets(&mut self) -> usize {
        self.streams
            .borrow()
            .streams
            .values()
            .map(|stream| stream.rtp_packets.len())
            .sum()
    }

    fn draw_points(&mut self, plot_ui: &mut PlotUi) {
        for point_data in &self.points_data {
            let PointData {
                x,
                y,
                height,
                on_hover,
                color,
                radius,
            } = point_data;
            plot_ui.line(
                Line::new(PlotPoints::new(vec![[*x, *y], [*x, y + height]]))
                    .name(on_hover)
                    .color(*color)
                    .width(0.5),
            );
            let point = Points::new([*x, *y])
                .name(on_hover)
                .color(*color)
                .radius(*radius);
            plot_ui.points(point);
        }
    }

    fn refresh_points(&mut self) {
        self.points_data.clear();
        let streams = self.streams.borrow();
        let mut points_x_and_y_plus_height: Vec<(f64, f64)> = Vec::new();
        let mut previous_stream_max_y = 0.0;

        streams
            .streams
            .iter()
            .enumerate()
            .for_each(|(stream_ix, (ssrc, stream))| {
                if !*(is_stream_visible(&mut self.streams_visibility, *ssrc)) {
                    return;
                }

                build_stream_points(
                    &streams,
                    &mut points_x_and_y_plus_height,
                    stream_ix,
                    &stream.rtp_packets,
                    stream.display_name.to_string(),
                    self.settings_x_axis,
                    &mut self.points_data,
                    previous_stream_max_y * 1.2,
                    &mut previous_stream_max_y,
                );
            });
    }
}

fn build_stream_points(
    streams: &Ref<Streams>,
    points_x_and_y_plus_height: &mut Vec<(f64, f64)>,
    stream_ix: usize,
    rtp_packets: &[usize],
    display_name: String,
    settings_x_axis: SettingsXAxis,
    points_data: &mut Vec<PointData>,
    this_stream_y_baseline: f64,
    previous_stream_max_y: &mut f64,
) {
    if rtp_packets.is_empty() {
        return;
    }

    let first_rtp_id = rtp_packets.first().unwrap();
    let first_packet = streams.packets.get(*first_rtp_id).unwrap();
    let SessionPacket::Rtp(ref first_rtp_packet) = first_packet.contents else {
        unreachable!();
    };

    rtp_packets.iter().enumerate().for_each(|(packet_ix, &ix)| {
        let packet = streams.packets.get(ix).unwrap();
        let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
            unreachable!();
        };

        let previous_packet = if packet_ix == 0 {
            None
        } else {
            let prev_rtp_id = *rtp_packets.get(packet_ix - 1).unwrap();
            streams.packets.get(prev_rtp_id)
        };

        let (x, y, height) = get_x_and_y(
            points_x_and_y_plus_height,
            stream_ix,
            first_rtp_packet,
            previous_packet,
            packet,
            rtp_packet,
            settings_x_axis,
            this_stream_y_baseline,
        );
        let on_hover = build_on_hover_text(
            display_name.to_string(),
            packet,
            rtp_packet,
            x,
            settings_x_axis,
        );

        points_data.push(PointData {
            x,
            y,
            height,
            on_hover,
            color: get_color(rtp_packet),
            radius: get_radius(rtp_packet),
        });

        let y_and_height = y + height;
        if *previous_stream_max_y < y_and_height {
            *previous_stream_max_y = y_and_height;
        }

        points_x_and_y_plus_height.push((x, y_and_height));
    });
}

fn get_x_and_y(
    points_x_and_y_plus_height: &mut [(f64, f64)],
    stream_ix: usize,
    first_rtp_packet: &RtpPacket,
    previous_packet: Option<&Packet>,
    packet: &Packet,
    rtp_packet: &RtpPacket,
    settings_x_axis: SettingsXAxis,
    this_stream_y_baseline: f64,
) -> (f64, f64, f64) {
    let (x, y, height) = match settings_x_axis {
        RtpTimestamp => {
            let minimum_shift = 0.02;
            let payload_length_shift = rtp_packet.payload_length as f64;
            let height = minimum_shift * payload_length_shift;

            if let Some(prev_packet) = previous_packet {
                let SessionPacket::Rtp(ref prev_rtp) = prev_packet.contents else {
                    unreachable!();
                };

                let y = if rtp_packet.timestamp != prev_rtp.timestamp {
                    this_stream_y_baseline
                } else {
                    let prev_y_plus_height =
                        points_x_and_y_plus_height.last().unwrap().to_owned().1;
                    prev_y_plus_height
                };

                (
                    rtp_packet.timestamp as f64 - first_rtp_packet.timestamp as f64,
                    y,
                    height,
                )
            } else {
                (
                    rtp_packet.timestamp as f64 - first_rtp_packet.timestamp as f64,
                    this_stream_y_baseline,
                    height,
                )
            }
        }
        RawTimestamp => (packet.timestamp.as_secs_f64(), stream_ix as f64, 0.0),
        SequenceNumer => (
            (rtp_packet.sequence_number - first_rtp_packet.sequence_number) as f64,
            stream_ix as f64,
            0.0,
        ),
    };
    (x, y, height)
}

fn build_on_hover_text(
    display_name: String,
    packet: &Packet,
    rtp_packet: &RtpPacket,
    x: f64,
    settings_x_axis: SettingsXAxis,
) -> String {
    let mut on_hover = String::new();

    on_hover.push_str(&format!(
        "Alias: {} (SSRC: {})",
        display_name, rtp_packet.ssrc
    ));
    on_hover.push('\n');
    on_hover.push_str(&format!(
        "Source: {}\nDestination: {}\n",
        packet.source_addr, packet.destination_addr
    ));
    if rtp_packet.previous_packet_is_lost {
        on_hover.push_str("\n***Previous packet is lost!***\n")
    }
    let marker_info = if rtp_packet.marker {
        match rtp_packet.payload_type.media_type {
            MediaType::Audio => {
                "\nFor audio payload type, marker says that it is first packet after silence.\n"
            }
            MediaType::Video => {
                "\nFor video payload type, marker says that it is last packet of a video frame.\n"
            }
            MediaType::AudioOrVideo => {
                "\nMarker could say that it is last packet of a video frame or \n\
                     that it is a first packet after silence.\n"
            }
        }
    } else {
        "".as_str()
    };
    on_hover.push_str(marker_info);
    on_hover.push('\n');
    on_hover.push_str(&format!("Sequence number: {}", rtp_packet.sequence_number));
    on_hover.push('\n');
    on_hover.push_str(&format!("Payload length: {}", rtp_packet.payload_length));
    on_hover.push('\n');
    on_hover.push_str(&format!("Padding: {}", rtp_packet.padding));
    on_hover.push('\n');
    on_hover.push_str(&format!("Extensions headers: {}", rtp_packet.extension));
    on_hover.push('\n');
    on_hover.push_str(&format!("Marker: {}", rtp_packet.marker));
    on_hover.push('\n');
    on_hover.push_str(&format!("CSRC: {:?}", rtp_packet.csrc));
    on_hover.push('\n');
    on_hover.push_str(&rtp_packet.payload_type.to_string());
    on_hover.push('\n');
    on_hover.push_str(&format!("x = {} [{}]\n", x, settings_x_axis));
    on_hover
}

fn get_radius(rtp_packet: &RtpPacket) -> f32 {
    if rtp_packet.previous_packet_is_lost {
        2.5
    } else {
        1.5
    }
}

fn get_color(rtp_packet: &RtpPacket) -> Color32 {
    if rtp_packet.previous_packet_is_lost {
        Color32::GOLD
    } else if rtp_packet.marker {
        Color32::GREEN
    } else {
        Color32::RED
    }
}
