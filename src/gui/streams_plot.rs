use crate::analysis::rtp::{Streams};
use crate::sniffer::rtp::RtpPacket;
use eframe::egui;
use eframe::egui::{Context, Ui};
use egui::Window;

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

        Self {
            streams,
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

    fn ui(&mut self, _ui: &mut Ui, _ctx: &Context) {
    }
}
