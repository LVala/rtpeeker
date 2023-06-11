use eframe::egui;
use eframe::egui::plot::{Line, MarkerShape, Plot, PlotPoints, Points};
use eframe::egui::{Context, Ui};
use eframe::epaint::Color32;
use eframe::epaint::shape_transform::adjust_colors;
use egui::Window;
use pcap::Packet;

use crate::analysis::rtp::{Stream, Streams};
use crate::sniffer::rtp::RtpPacket;

struct MyPoints {
    normal_points: PlotPoints,
    with_marker_points: PlotPoints,
}
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StreamsPlot<'a> {
    pub streams: Streams<'a>,
}

impl<'a> StreamsPlot<'a> {
    pub fn new(rtp_packets: &'a [RtpPacket]) -> Self {
        let mut streams = Streams::new();

        for packet in rtp_packets {
            streams.add_packet(packet);
        }

        Self { streams }
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
        let mut normal_points: Vec<[f64; 2]> = Vec::new();
        let mut marker_points: Vec<[f64; 2]> = Vec::new();

        for stream_ix in 0..self.streams.streams.len() {
            let stream = self.streams.streams.get(stream_ix).unwrap();

            for packet in &stream.packets {
                if packet.packet.header.marker {
                    marker_points.push([packet.raw_packet.timestamp.as_secs_f64(), stream_ix as f64])
                } else {
                    normal_points.push([packet.raw_packet.timestamp.as_secs_f64(), stream_ix as f64])
                }
            }
        }

        let normal_points = Points::new(PlotPoints::new(normal_points))
            .color(Color32::DARK_RED)
            .filled(true)
            .radius(3.0);
        let marker_points = Points::new(PlotPoints::new(marker_points))
            .color(Color32::DARK_GREEN)
            .shape(MarkerShape::Diamond)
            .filled(true)
            .radius(3.5);


        Plot::new("halo")
            .view_aspect(2.0)
            .show(ui, |plot_ui| {
                plot_ui.points(normal_points);
                plot_ui.points(marker_points);
            });
    }
}
