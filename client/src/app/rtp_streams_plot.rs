use self::SettingsXAxis::*;
use super::is_stream_visible;
use crate::streams::stream::{RtpInfo, Stream};
use crate::streams::{RefStreams, Streams};
use eframe::egui;
use eframe::egui::TextBuffer;
use eframe::epaint::Color32;
use egui::plot::{
    Line, LineStyle, MarkerShape, Plot, PlotBounds, PlotPoint, PlotPoints, PlotUi, Points, Text,
};
use egui::Ui;
use egui::{Align2, RichText};
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::rtcp::ReceptionReport;
use rtpeeker_common::rtp::payload_type::MediaType;
use rtpeeker_common::StreamKey;
use rtpeeker_common::{Packet, RtcpPacket, RtpPacket};
use std::cell::Ref;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};

struct PointData {
    x: f64,
    y_low: f64,
    y_top: f64,
    on_hover: String,
    color: Color32,
    radius: f32,
    is_rtcp: bool,
    marker_shape: MarkerShape,
}

struct StreamSeperatorLine {
    x_start: f64,
    x_end: f64,
    y: f64,
}

struct StreamText {
    x: f64,
    y: f64,
    on_hover: String,
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
            RawTimestamp => "Seconds from start",
            SequenceNumer => "Sequence number",
        };

        write!(f, "{}", name)
    }
}

pub struct RtpStreamsPlot {
    streams: RefStreams,
    points_data: Vec<PointData>,
    stream_separator_lines: Vec<StreamSeperatorLine>,
    stream_texts: Vec<StreamText>,
    x_axis: SettingsXAxis,
    requires_reset: bool,
    streams_visibility: HashMap<StreamKey, bool>,
    last_rtp_packets_len: usize,
    set_plot_bounds: bool,
    slider_max: i64,
    slider_start: i64,
    slider_length: i64,
    first_draw: bool,
}

impl RtpStreamsPlot {
    pub fn new(streams: RefStreams) -> Self {
        Self {
            streams,
            points_data: Vec::new(),
            stream_separator_lines: Vec::new(),
            stream_texts: Vec::new(),
            x_axis: RtpTimestamp,
            requires_reset: false,
            streams_visibility: HashMap::default(),
            last_rtp_packets_len: 0,
            set_plot_bounds: false,
            slider_max: 10000,
            slider_start: 0,
            slider_length: 1,
            first_draw: true,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.collapsing("Help", |ui| {
                    Self::build_help_section(ui);
                });
                ui.collapsing("Settings", |ui| {
                    self.options_ui(ui);
                });
            });
            self.plot_ui(ui);
        });
    }

    fn build_help_section(ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(0.8);
                    ui.label(RichText::from("     🔴").color(Color32::RED));
                    ui.label("End of a packet");
                });
                ui.label(RichText::from("       |").color(Color32::RED));
                ui.horizontal(|ui| {
                    ui.label(RichText::from("       |").color(Color32::RED));
                    ui.label("  Length of line represents payload size relative to other packets.");
                });

                ui.label(RichText::from("       |").color(Color32::RED));
                ui.horizontal(|ui| {
                    ui.label(RichText::from("       |").color(Color32::RED));
                    ui.label("  Beginning of a packet");
                });
            });
            ui.vertical(|ui| {
                ui.label("The color of a dot presents:");
                ui.horizontal(|ui| {
                    ui.label(RichText::from("\t🔴").color(Color32::RED));
                    ui.label("Ordinary RTP packet");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::from("\t🔴").color(Color32::GREEN));
                    ui.label("RTP packet has marker");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::from("\t🔴").color(Color32::GOLD));
                    ui.label("At least one previous packet is lost");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::from("\t■").color(Color32::from_rgb(200, 0, 200)));
                    ui.label("RTCP packet");
                });
            })
        });
    }

    fn options_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.vertical(|ui| {
                self.reset_button(ui);
                self.axis_settings(ui);
                self.toggle_streams(ui);
            });
            ui.separator();
            ui.vertical(|ui| {
                self.plot_bounds_ui_options(ui);
            });
        });
    }

    fn plot_bounds_ui_options(&mut self, ui: &mut Ui) {
        let set_plot_button_clicked = ui.button("Set plot bounds").clicked();

        let (x_min_text, x_max_text) = match self.x_axis {
            RtpTimestamp => ("First RTP timestamp", "Length"),
            RawTimestamp => ("First second", "Length"),
            SequenceNumer => ("First sequence number", "Length"),
        };

        let max = (self.slider_max as f64 * 1.13) as i64;
        let x_min_resp =
            ui.add(egui::Slider::new(&mut self.slider_start, 0..=max).text(x_min_text));
        let x_max_resp =
            ui.add(egui::Slider::new(&mut self.slider_length, 1..=max).text(x_max_text));

        if set_plot_button_clicked | x_min_resp.dragged() | x_max_resp.dragged() {
            self.set_plot_bounds = true
        }
        ui.vertical(|ui| {
            ui.add_space(10.0);
        });
    }

    fn toggle_streams(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            let mut aliases = Vec::new();
            let streams = &self.streams.borrow().streams;
            let keys: Vec<_> = streams.keys().collect();

            keys.iter().for_each(|&key| {
                let alias = streams.get(key).unwrap().alias.to_string();
                aliases.push((*key, alias));
            });
            aliases.sort_by(|(_, a), (_, b)| a.cmp(b));

            ui.label(RichText::from("Toggle streams: ").strong());
            aliases.iter().for_each(|(key, alias)| {
                let selected = is_stream_visible(&mut self.streams_visibility, *key);
                if ui.checkbox(selected, alias).clicked() {
                    self.requires_reset = true
                }
            });
        });
    }

    fn axis_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::from("X axis:").strong());
            SettingsXAxis::all().into_iter().for_each(|setting| {
                if ui
                    .radio(setting == self.x_axis, setting.to_string())
                    .clicked()
                {
                    self.x_axis = setting;
                    self.slider_max = 1;
                    self.slider_length = 1;
                    self.slider_start = 0;
                    self.requires_reset = true;
                }
            });
        });
    }

    fn reset_button(&mut self, ui: &mut Ui) {
        if ui.button("Reset to initial state").clicked() {
            self.requires_reset = true;
        }
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
        let mut heighest_y = 0.0;
        for point_data in &self.points_data {
            let PointData {
                x,
                y_low,
                y_top,
                on_hover,
                color,
                radius,
                is_rtcp,
                marker_shape,
            } = point_data;
            if *y_top > heighest_y {
                heighest_y = *y_top;
            }

            let point = Points::new([*x, *y_top])
                .name(on_hover)
                .color(*color)
                .radius(*radius)
                .shape(*marker_shape);

            plot_ui.points(point);
            if !is_rtcp {
                plot_ui.line(
                    Line::new(PlotPoints::new(vec![[*x, *y_low], [*x, *y_top]]))
                        .color(Color32::RED)
                        .highlight(false)
                        .width(0.5),
                );
            }
        }
        for separator in &self.stream_separator_lines {
            let StreamSeperatorLine { x_start, x_end, y } = separator;
            plot_ui.line(
                Line::new(PlotPoints::new(vec![[*x_start, *y], [*x_end, *y]]))
                    .color(Color32::GRAY)
                    .style(LineStyle::Solid)
                    .width(0.5),
            );
        }

        for text in &self.stream_texts {
            let StreamText { x, y, on_hover } = text;
            plot_ui.text(
                Text::new(
                    PlotPoint { x: *x, y: *y },
                    RichText::new(on_hover)
                        .color(Color32::LIGHT_GRAY)
                        .strong()
                        .size(12.0),
                )
                .anchor(Align2::RIGHT_TOP),
            )
        }

        if !self.first_draw && self.set_plot_bounds {
            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                [(self.slider_start as f64) - 0.05, -0.5],
                [
                    (self.slider_start + self.slider_length) as f64,
                    heighest_y * 1.55,
                ],
            ));
            self.set_plot_bounds = false
        }
        if self.first_draw {
            self.first_draw = false
        }
    }

    fn refresh_points(&mut self) {
        self.points_data.clear();
        self.stream_separator_lines.clear();
        self.stream_texts.clear();
        let streams = self.streams.borrow();
        let mut points_x_and_y_top: Vec<(f64, f64)> = Vec::new();
        let mut previous_stream_max_y = 0.0;
        let mut biggest_margin = 0.0;
        let mut previous_stream_height = 0.0;

        streams
            .streams
            .iter()
            .enumerate()
            .for_each(|(i, (key, stream))| {
                if !*(is_stream_visible(&mut self.streams_visibility, *key)) {
                    return;
                }
                if i != 0 {
                    let stream_highest_y_if_drawn = get_highest_y(
                        &streams,
                        &mut Vec::new(),
                        stream,
                        self.x_axis,
                        previous_stream_max_y,
                    );
                    let stream_height_if_drawn = stream_highest_y_if_drawn - previous_stream_max_y;
                    let margin = stream_height_if_drawn * 0.2 + previous_stream_height * 0.2;
                    if biggest_margin < margin {
                        biggest_margin = margin;
                    }
                };

                let this_stream_y_baseline = match self.x_axis {
                    RtpTimestamp => previous_stream_max_y + biggest_margin,
                    RawTimestamp => previous_stream_max_y + 20.0,
                    SequenceNumer => previous_stream_max_y + 20.0,
                };
                self.stream_texts.push(StreamText {
                    x: 0.0,
                    y: this_stream_y_baseline,
                    on_hover: String::from(&format!("{} ({:x})  ", stream.alias, stream.ssrc)),
                });
                if let Some((x, _)) = points_x_and_y_top.last() {
                    self.stream_separator_lines.push(StreamSeperatorLine {
                        x_start: 0.0,
                        x_end: *x,
                        y: (this_stream_y_baseline + previous_stream_max_y) / 2.0,
                    })
                };

                build_stream_points(
                    &streams,
                    &mut points_x_and_y_top,
                    stream,
                    self.x_axis,
                    &mut self.points_data,
                    &mut previous_stream_max_y,
                    &mut self.slider_max,
                    this_stream_y_baseline,
                );
                previous_stream_height = previous_stream_max_y - this_stream_y_baseline;
            });
    }
}

fn get_highest_y(
    streams: &Ref<Streams>,
    points_x_and_y_top: &mut Vec<(f64, f64)>,
    stream: &Stream,
    settings_x_axis: SettingsXAxis,
    this_stream_y_baseline: f64,
) -> f64 {
    let mut highest_y = 0.0;
    let rtp_packets = &stream.rtp_packets;
    if rtp_packets.is_empty() {
        return highest_y;
    }

    let first_rtp_id = rtp_packets.first().unwrap();
    let first_packet = streams.packets.get(first_rtp_id.id).unwrap();
    let SessionPacket::Rtp(ref first_rtp_packet) = first_packet.contents else {
        unreachable!();
    };

    rtp_packets
        .iter()
        .enumerate()
        .for_each(|(packet_ix, packet)| {
            let previous_packet = if packet_ix == 0 {
                None
            } else {
                let prev_rtp_id = rtp_packets.get(packet_ix - 1).unwrap().id;
                streams.packets.get(prev_rtp_id)
            };

            let (_, _, y_top) = get_x_and_y(
                points_x_and_y_top,
                first_rtp_packet,
                previous_packet,
                packet,
                settings_x_axis,
                this_stream_y_baseline,
                first_packet,
            );
            points_x_and_y_top.push((y_top, y_top));

            if highest_y < y_top {
                highest_y = y_top;
            }
        });
    highest_y
}

#[allow(clippy::too_many_arguments)]
fn build_stream_points(
    streams: &Ref<Streams>,
    points_x_and_y_top: &mut Vec<(f64, f64)>,
    stream: &Stream,
    settings_x_axis: SettingsXAxis,
    points_data: &mut Vec<PointData>,
    previous_stream_max_y: &mut f64,
    slider_max: &mut i64,
    this_stream_y_baseline: f64,
) {
    let rtp_packets = &stream.rtp_packets;
    let rtcp_packets = &stream.rtcp_packets;
    if rtp_packets.is_empty() {
        return;
    }

    let first_rtp_id = rtp_packets.first().unwrap();
    let first_packet = streams.packets.get(first_rtp_id.id).unwrap();
    let SessionPacket::Rtp(ref first_rtp_packet) = first_packet.contents else {
        unreachable!();
    };

    let mut on_hover = String::new();
    let mut maybe_prev_id = None;

    rtcp_packets.iter().for_each(|rtcp_info| {
        match &rtcp_info.packet {
            RtcpPacket::SenderReport(sender_report) => {
                on_hover.push_str("Sender Report\n\n");
                on_hover.push_str(&format!("Source: {:x}\n", sender_report.ssrc));
                on_hover.push_str(&format!("NTP time: {}\n", sender_report.ntp_time));
                on_hover.push_str(&format!("RTP time: {}\n", sender_report.rtp_time));
                for report in &sender_report.reports {
                    build_reception_report(&mut on_hover, &report);
                }
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::ReceiverReport(receiver_report) => {
                on_hover.push_str("Receiver Report\n\n");
                on_hover.push_str(&format!("Source: {:x}\n", receiver_report.ssrc));
                for report in &receiver_report.reports {
                    build_reception_report(&mut on_hover, &report);
                }
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::SourceDescription(source_description) => {
                on_hover.push_str("Source Description\n\n");
                for chunk in &source_description.chunks {
                    on_hover.push_str(&format!("Source: {:x}\n", chunk.source));
                    for item in &chunk.items {
                        on_hover.push_str(&format!("{}: {}\n", item.sdes_type, item.text));
                    }
                }
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::Goodbye(goodbye) => {
                on_hover.push_str("Goodbye\n\n");
                for source in &goodbye.sources {
                    on_hover.push_str(&format!("Source: {:x}", source));
                    on_hover.push_str(&format!("Reason: {}", goodbye.reason));
                }
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::ApplicationDefined => {
                on_hover.push_str("\nGoodbye\n");
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::PayloadSpecificFeedback => {
                on_hover.push_str("\nPayload specific feedback\n");
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::TransportSpecificFeedback => {
                on_hover.push_str("\nTransport specific feedback\n");
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::ExtendedReport => {
                on_hover.push_str("\nExtended report\n");
                on_hover.push_str("------------------------\n");
            }
            RtcpPacket::Other => {
                on_hover.push_str("\nOther rtcp\n");
                on_hover.push_str("------------------------\n");
            }
        };
        let x = rtcp_info.time.as_secs_f64() - first_packet.timestamp.as_secs_f64();
        let y = if let Some(last_position) = points_x_and_y_top.last() {
            let last_x = last_position.0;
            let last_y_top = last_position.1;
            if x == last_x {
                let shift = 3.0;
                last_y_top + shift
            } else {
                this_stream_y_baseline
            }
        } else {
            this_stream_y_baseline
        };
        if *previous_stream_max_y < y {
            *previous_stream_max_y = y;
        }

        if let Some(prev_id) = maybe_prev_id {
            if prev_id != rtcp_info.id {
                match settings_x_axis {
                    RtpTimestamp => {}
                    RawTimestamp => {
                        let saved_on_hover = on_hover.clone();
                        points_data.push(PointData {
                            x,
                            y_low: y,
                            y_top: y,
                            on_hover: saved_on_hover,
                            color: Color32::from_rgb(200, 0, 200),
                            radius: 3.5,
                            is_rtcp: true,
                            marker_shape: MarkerShape::Square,
                        });
                        on_hover = String::new();
                        points_x_and_y_top.push((x, y));
                        let x = x as i64;
                        if x > *slider_max {
                            *slider_max = x;
                        }
                    }
                    SequenceNumer => {}
                }
            }
        }
        maybe_prev_id = Some(rtcp_info.id);
    });
    rtp_packets
        .iter()
        .enumerate()
        .for_each(|(packet_ix, packet)| {
            let previous_packet = if packet_ix == 0 {
                None
            } else {
                let prev_rtp_id = rtp_packets.get(packet_ix - 1).unwrap().id;
                streams.packets.get(prev_rtp_id)
            };

            let (x, y_low, y_top) = get_x_and_y(
                points_x_and_y_top,
                first_rtp_packet,
                previous_packet,
                packet,
                settings_x_axis,
                this_stream_y_baseline,
                first_packet,
            );
            let on_hover = build_on_hover_text(stream, packet, x, settings_x_axis);

            points_data.push(PointData {
                x,
                y_low,
                y_top,
                on_hover,
                color: get_color(packet),
                radius: get_radius(packet),
                is_rtcp: false,
                marker_shape: MarkerShape::Circle,
            });

            if *previous_stream_max_y < y_top {
                *previous_stream_max_y = y_top;
            }

            points_x_and_y_top.push((x, y_top));
            let x = x as i64;
            if x > *slider_max {
                *slider_max = x;
            }
        });
}

fn build_reception_report(on_hover: &mut String, report: &&ReceptionReport) {
    on_hover.push_str(&"-".repeat(160));
    on_hover.push('\n');
    on_hover.push_str(&format!("SSRC: {:x}\t", report.ssrc));
    on_hover.push_str(&format!("Fraction lost: {}\t", report.fraction_lost));
    on_hover.push_str(&format!("Cumulative lost: {}\t", report.total_lost));
    on_hover.push_str(&format!(
        "Extended highest sequence number: {}\n",
        report.last_sequence_number
    ));
    on_hover.push_str(&format!("Interarrival jitter: {}\t", report.jitter));
    on_hover.push_str(&format!(
        "Last SR timestamp: {}\t",
        report.last_sender_report
    ));
    on_hover.push_str(&format!("Delay since last SR: {}\n", report.delay));
}

#[allow(clippy::too_many_arguments)]
fn get_x_and_y(
    points_x_and_y_top: &mut [(f64, f64)],
    first_rtp_packet: &RtpPacket,
    previous_packet: Option<&Packet>,
    rtp: &RtpInfo,
    settings_x_axis: SettingsXAxis,
    this_stream_y_baseline: f64,
    first_packet: &Packet,
) -> (f64, f64, f64) {
    let minimum_shift = 0.02;
    let payload_length_shift = rtp.packet.payload_length as f64;
    let height = minimum_shift * payload_length_shift;

    let (x, y, y_top) = match settings_x_axis {
        RtpTimestamp => {
            if let Some(prev_packet) = previous_packet {
                let SessionPacket::Rtp(ref prev_rtp) = prev_packet.contents else {
                    unreachable!();
                };

                let last_y_top = if rtp.packet.timestamp != prev_rtp.timestamp {
                    this_stream_y_baseline
                } else {
                    let prev_y_top = points_x_and_y_top.last().unwrap().to_owned().1;
                    prev_y_top
                };

                (
                    rtp.packet.timestamp as f64 - first_rtp_packet.timestamp as f64,
                    last_y_top,
                    last_y_top + height,
                )
            } else {
                (
                    rtp.packet.timestamp as f64 - first_rtp_packet.timestamp as f64,
                    this_stream_y_baseline,
                    this_stream_y_baseline + height,
                )
            }
        }
        RawTimestamp => (
            rtp.time.as_secs_f64() - first_packet.timestamp.as_secs_f64(),
            this_stream_y_baseline,
            this_stream_y_baseline + height,
        ),
        SequenceNumer => (
            (rtp.packet.sequence_number - first_rtp_packet.sequence_number) as f64,
            this_stream_y_baseline,
            this_stream_y_baseline + height,
        ),
    };
    (x, y, y_top)
}

fn build_on_hover_text(
    stream: &Stream,
    rtp: &RtpInfo,
    x: f64,
    settings_x_axis: SettingsXAxis,
) -> String {
    let mut on_hover = String::new();

    on_hover.push_str(&format!(
        "Alias: {} (SSRC: {:x})",
        stream.alias, stream.ssrc
    ));
    on_hover.push('\n');
    on_hover.push_str(&format!(
        "Source: {}\nDestination: {}\n",
        stream.source_addr, stream.destination_addr
    ));
    if rtp.prev_lost {
        on_hover.push_str("\n***Previous packet is lost!***\n")
    }
    let marker_info = if rtp.packet.marker {
        match rtp.packet.payload_type.media_type {
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
    on_hover.push_str(&format!("Sequence number: {}", rtp.packet.sequence_number));
    on_hover.push('\n');
    on_hover.push_str(&format!("Payload length: {}", rtp.packet.payload_length));
    on_hover.push('\n');
    on_hover.push_str(&format!("Padding: {}", rtp.packet.padding));
    on_hover.push('\n');
    on_hover.push_str(&format!("Extensions headers: {}", rtp.packet.extension));
    on_hover.push('\n');
    on_hover.push_str(&format!("Marker: {}", rtp.packet.marker));
    on_hover.push('\n');
    on_hover.push_str(&format!("CSRC: {:?}", rtp.packet.csrc));
    on_hover.push('\n');
    on_hover.push_str(&rtp.packet.payload_type.to_string());
    on_hover.push('\n');
    let str = match settings_x_axis {
        RtpTimestamp => format!("x = {} [RTP timestamp]\n", x),
        RawTimestamp => format!("x = {:.5} [s]\n", x),
        SequenceNumer => format!("x = {} [Sequence number]\n", x),
    };
    on_hover.push_str(&str);
    on_hover
}

fn get_radius(rtp: &RtpInfo) -> f32 {
    if rtp.prev_lost {
        3.5
    } else if rtp.packet.marker {
        2.5
    } else {
        2.0
    }
}

fn get_color(rtp: &RtpInfo) -> Color32 {
    if rtp.prev_lost {
        Color32::GOLD
    } else if rtp.packet.marker {
        Color32::GREEN
    } else {
        Color32::RED
    }
}
