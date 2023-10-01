use std::cell::Ref;
use std::fmt::{Display, Error, Formatter};

use eframe::egui;
use eframe::egui::TextBuffer;
use eframe::epaint::Color32;
use egui::epaint::ahash::HashMap;
use egui::plot::{Plot, PlotUi, Points};
use egui::Ui;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::rtp::payload_type::MediaType;
use rtpeeker_common::{Packet, RtpPacket};

use crate::streams::{is_stream_visible, RefStreams, Streams};

use self::SettingsXAxis::*;

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
    points_data: Vec<(f64, f64, String, Color32, f32)>,
    settings_x_axis: SettingsXAxis,
    requires_reset: bool,
    streams_visibility: HashMap<u32, bool>,
    last_packets_len: usize,
}

impl RtpStreamsPlot {
    pub fn new(streams: RefStreams) -> Self {
        Self {
            streams,
            points_data: Vec::new(),
            settings_x_axis: RtpTimestamp,
            requires_reset: false,
            streams_visibility: HashMap::default(),
            last_packets_len: 0,
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
        let number_of_packets = self.streams.borrow().packets.len();
        if self.last_packets_len != number_of_packets {
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
        self.last_packets_len = number_of_packets;
    }

    fn draw_points(&mut self, plot_ui: &mut PlotUi) {
        for (x, y, on_hover, color, radius) in &self.points_data {
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
        let mut points_xy: Vec<(f64, f64)> = Vec::new();

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
                    &mut points_xy,
                    stream_ix,
                    &stream.rtp_packets,
                    stream.display_name.to_string(),
                    self.settings_x_axis,
                    &mut self.points_data,
                );
            });
    }
}

fn build_stream_points(
    streams: &Ref<Streams>,
    points_xy: &mut Vec<(f64, f64)>,
    stream_ix: usize,
    rtp_packets: &[usize],
    display_name: String,
    settings_x_axis: SettingsXAxis,
    points_data: &mut Vec<(f64, f64, String, Color32, f32)>,
) {
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

        let (x, y) = get_x_and_y(
            points_xy,
            stream_ix,
            previous_packet,
            packet,
            rtp_packet,
            settings_x_axis,
        );
        let on_hover = build_on_hover_text(
            display_name.to_string(),
            packet,
            rtp_packet,
            x,
            settings_x_axis,
        );

        points_data.push((x, y, on_hover, get_color(rtp_packet), get_radius()));
        points_xy.push((x, y));
    });
}

fn get_x_and_y(
    points_xy: &mut [(f64, f64)],
    stream_ix: usize,
    previous_packet: Option<&Packet>,
    packet: &Packet,
    rtp_packet: &RtpPacket,
    settings_x_axis: SettingsXAxis,
) -> (f64, f64) {
    let (x, y) = match settings_x_axis {
        RtpTimestamp => {
            if let Some(prev_packet) = previous_packet {
                let SessionPacket::Rtp(ref prev_rtp) = prev_packet.contents else {
                    unreachable!();
                };

                let y = if rtp_packet.timestamp != prev_rtp.timestamp {
                    stream_ix as f64
                } else {
                    let prev_y = points_xy.last().unwrap().to_owned().1;
                    let y_shift = 0.01;
                    prev_y + y_shift
                };

                (rtp_packet.timestamp as f64, y)
            } else {
                (rtp_packet.timestamp as f64, stream_ix as f64)
            }
        }
        RawTimestamp => (packet.timestamp.as_secs_f64(), stream_ix as f64),
        SequenceNumer => (rtp_packet.sequence_number as f64, stream_ix as f64),
    };
    (x, y)
}

fn build_on_hover_text(
    display_name: String,
    packet: &Packet,
    rtp_packet: &RtpPacket,
    x: f64,
    settings_x_axis: SettingsXAxis,
) -> String {
    let mut on_hover = String::new();

    on_hover.push_str(&display_name);
    on_hover.push('\n');
    on_hover.push_str(&format!(
        "Source: {}\nDestination: {}\n",
        packet.source_addr, packet.destination_addr
    ));
    on_hover.push('\n');
    on_hover.push_str(&rtp_packet.payload_type.to_string());
    on_hover.push('\n');
    let additional_info = if rtp_packet.marker {
        match rtp_packet.payload_type.media_type {
            MediaType::Audio => {
                "For audio payload type, marker says that it is first packet after silence.\n"
            }
            MediaType::Video => {
                "For video payload type, marker says that it is last packet of a video frame.\n"
            }
            MediaType::AudioOrVideo => {
                "Marker could say that it is last packet of a video frame or \n\
                     that it is a first packet after silence.\n"
            }
        }
    } else {
        "".as_str()
    };
    on_hover.push_str(additional_info);
    on_hover.push_str(&format!("x = {} [{}]\n", x, settings_x_axis));
    on_hover
}

fn get_radius() -> f32 {
    1.5
}

fn get_color(rtp_packet: &RtpPacket) -> Color32 {
    if rtp_packet.marker {
        Color32::GREEN
    } else {
        Color32::RED
    }
}
